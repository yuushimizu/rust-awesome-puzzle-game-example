pub mod block;
pub mod event;
pub mod piece;
mod piece_generator;

pub use block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace};
pub use event::GameEvent;
pub use piece::Piece;

use piece_generator::PieceGenerator;
use std::iter;

const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const WAIT: f64 = 0.2;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PieceState {
    pub piece: Piece,
    pub position: BlockIndexOffset,
}

impl PieceState {
    pub fn new(piece: Piece, position: BlockIndexOffset) -> Self {
        Self { piece, position }
    }

    pub fn with_initial_position(piece: Piece, stage_size: BlockGridSize) -> Self {
        let position = BlockIndexOffset::new(
            ((stage_size.width - piece.size().width) / 2) as isize,
            stage_size.height as isize - piece.size().height as isize / 2,
        );
        Self::new(piece, position)
    }

    fn blocks<'a>(&'a self) -> impl iter::Iterator<Item = (BlockIndexOffset, Block)> + 'a {
        self.piece
            .blocks()
            .map(move |(index, block)| (self.position + index.cast::<isize>().to_vector(), block))
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    stage: BlockGrid,
    piece_state: PieceState,
    piece_generator: PieceGenerator,
    wait: f64,
}

impl Game {
    pub fn new() -> Self {
        let mut piece_generator = PieceGenerator::new(piece::standards());
        let stage_size = BlockGridSize::new(WIDTH, HEIGHT);
        Self {
            stage: BlockGrid::new(stage_size, None),
            piece_state: PieceState::with_initial_position(piece_generator.next(), stage_size),
            piece_generator,
            wait: WAIT,
        }
    }

    pub fn stage_size(&self) -> BlockGridSize {
        self.stage.size()
    }

    fn can_put_to(&self, index: BlockIndexOffset) -> bool {
        index.x >= 0
            && (index.x as usize) < self.stage_size().width
            && index.y >= 0
            && (index.y as usize >= self.stage_size().height
                || self.stage[index.cast::<usize>()].is_none())
    }

    fn can_transform_piece(
        &self,
        mut transform: impl FnMut(BlockIndexOffset) -> BlockIndexOffset,
    ) -> bool {
        self.piece_state
            .blocks()
            .map(move |(index, _)| transform(index))
            .all(|index| self.can_put_to(index))
    }

    fn can_move(&self, offset: euclid::TypedVector2D<isize, BlockSpace>) -> bool {
        self.can_transform_piece(|index| index + offset)
    }

    fn search_hard_drop_position(&self) -> BlockIndexOffset {
        let mut offset = euclid::TypedVector2D::new(0, 0);
        while self.can_move(offset + euclid::TypedVector2D::new(0, -1)) {
            offset.y -= 1;
        }
        self.piece_state.position + offset
    }

    fn change_piece_event(&self) -> GameEvent {
        GameEvent::ChangePiece {
            piece: self.piece_state.piece.clone(),
            guide_position: self.search_hard_drop_position(),
        }
    }

    fn move_piece_event(&self) -> GameEvent {
        GameEvent::MovePiece {
            piece: self.piece_state.piece.clone(),
            position: self.piece_state.position,
            guide_position: self.search_hard_drop_position(),
        }
    }

    fn update_next_pieces_event(&mut self) -> GameEvent {
        GameEvent::UpdateNextPieces(self.piece_generator.peek(3))
    }

    fn is_filled_line(&self, y: usize) -> bool {
        self.stage
            .line(y)
            .unwrap()
            .iter()
            .all(|block| block.is_some())
    }

    fn filled_line_indices(&self) -> Vec<usize> {
        (0..self.stage_size().height)
            .filter(|&y| self.is_filled_line(y))
            .collect::<Vec<_>>()
    }

    fn remove_lines(&mut self, indices: &[usize]) -> GameEvent {
        let stage_size = self.stage_size();
        let mut removed_blocks = vec![];
        for index in indices
            .iter()
            .flat_map(|&y| (0..stage_size.width).map(move |x| BlockIndex::new(x, y)))
        {
            removed_blocks.push((self.stage[index].unwrap(), index));
            self.stage[index] = None;
        }
        GameEvent::RemoveBlocks(removed_blocks)
    }

    fn remove_filled_lines(&mut self) -> Vec<GameEvent> {
        let stage_size = self.stage_size();
        let line_indices = self.filled_line_indices();
        let mut events = vec![self.remove_lines(&line_indices)];
        let mut moves = vec![];
        let mut line_boundaries = line_indices;
        line_boundaries.push(self.stage_size().height);
        for (removed, range) in line_boundaries
            .windows(2)
            .map(|area| (area[0] + 1..area[1]))
            .enumerate()
        {
            for index in
                range.flat_map(|y| (0..stage_size.width).map(move |x| BlockIndex::new(x, y)))
            {
                if let Some(block) = self.stage[index] {
                    let destination = BlockIndex::new(index.x, index.y - removed - 1);
                    moves.push((block, index, destination));
                    self.stage[destination] = std::mem::replace(&mut self.stage[index], None);
                }
            }
        }
        events.push(GameEvent::MoveBlocks(moves));
        events
    }

    fn fix_piece(&mut self) -> Vec<GameEvent> {
        let mut events = vec![GameEvent::RemovePiece];
        let mut blocks = vec![];
        for (index, block) in self.piece_state.blocks() {
            if (index.y as usize) < self.stage_size().height {
                let index = index.cast::<usize>();
                self.stage[index] = Some(block);
                blocks.push((block, index));
            }
        }
        events.push(GameEvent::PutBlocks(blocks));
        events.append(&mut self.remove_filled_lines());
        self.piece_state =
            PieceState::with_initial_position(self.piece_generator.next(), self.stage_size());
        events.push(self.change_piece_event());
        events.push(self.move_piece_event());
        events.push(self.update_next_pieces_event());
        events
    }

    pub fn initial_events(&mut self) -> Vec<GameEvent> {
        vec![
            self.change_piece_event(),
            self.move_piece_event(),
            self.update_next_pieces_event(),
        ]
    }

    pub fn update(&mut self, delta: f64) -> Vec<GameEvent> {
        if self.wait <= delta {
            self.wait = WAIT - (delta - self.wait);
            self.drop_piece_soft()
        } else {
            self.wait -= delta;
            vec![]
        }
    }

    fn try_move(&mut self, offset: isize) -> Vec<GameEvent> {
        if self.can_move(euclid::TypedVector2D::new(offset, 0)) {
            self.piece_state.position.x += offset;
            vec![self.move_piece_event()]
        } else {
            vec![]
        }
    }

    pub fn move_piece_left(&mut self) -> Vec<GameEvent> {
        self.try_move(-1)
    }

    pub fn move_piece_right(&mut self) -> Vec<GameEvent> {
        self.try_move(1)
    }

    pub fn drop_piece_soft(&mut self) -> Vec<GameEvent> {
        if self.can_move(euclid::TypedVector2D::new(0, -1)) {
            self.piece_state.position.y -= 1;
            vec![self.move_piece_event()]
        } else {
            self.fix_piece()
        }
    }

    pub fn drop_piece_hard(&mut self) -> Vec<GameEvent> {
        self.piece_state.position = self.search_hard_drop_position();
        self.fix_piece()
    }

    fn try_change_piece(&mut self, new_piece: Piece) -> Vec<GameEvent> {
        let new_state = PieceState::new(new_piece, self.piece_state.position);
        if new_state.blocks().all(|(index, _)| self.can_put_to(index)) {
            self.piece_state = new_state;
            vec![self.change_piece_event()]
        } else {
            vec![]
        }
    }

    pub fn rotate_piece_right(&mut self) -> Vec<GameEvent> {
        self.try_change_piece(self.piece_state.piece.rotate_right())
    }

    pub fn rotate_piece_left(&mut self) -> Vec<GameEvent> {
        self.try_change_piece(self.piece_state.piece.rotate_left())
    }
}