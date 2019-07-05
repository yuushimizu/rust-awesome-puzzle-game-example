pub mod block;
pub mod event;
pub mod piece;
mod piece_producer;

pub use block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace};
pub use event::Event;
pub use piece::Piece;

use euclid;

use piece_producer::PieceProducer;
use std::iter;
const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const WAIT: f64 = 0.8;

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
    piece_producer: PieceProducer,
    wait: f64,
}

impl Game {
    pub fn new() -> Self {
        let mut piece_producer = PieceProducer::new(piece::standards());
        let piece = piece_producer.next();
        let stage_size = BlockGridSize::new(WIDTH, HEIGHT);
        Self {
            stage: BlockGrid::new(stage_size, None),
            piece_state: PieceState::with_initial_position(piece, stage_size),
            piece_producer,
            wait: WAIT,
        }
    }

    pub fn stage_size(&self) -> BlockGridSize {
        self.stage.size()
    }

    fn change_piece_event(&self) -> Event {
        Event::ChangePiece(self.piece_state.piece.clone())
    }

    fn move_piece_event(&self) -> Event {
        Event::MovePiece(self.piece_state.piece.clone(), self.piece_state.position)
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

    fn remove_filled_lines(&mut self) -> Vec<Event> {
        let stage_size = self.stage_size();
        let removed_line_indices = self.filled_line_indices();
        let mut remove_lines = || {
            let mut removed_blocks = vec![];
            for index in (&removed_line_indices)
                .iter()
                .flat_map(|&y| (0..stage_size.width).map(move |x| BlockIndex::new(x, y)))
            {
                removed_blocks.push((self.stage[index].unwrap(), index));
                self.stage[index] = None;
            }
            Event::RemoveBlocks(removed_blocks)
        };
        let mut events = vec![remove_lines()];
        let mut moves = vec![];
        let mut line_boundaries = removed_line_indices;
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
        events.push(Event::MoveBlocks(moves));
        events
    }

    fn fix_piece(&mut self) -> Vec<Event> {
        let mut blocks = vec![];
        for (index, block) in self.piece_state.blocks() {
            if (index.y as usize) < self.stage_size().height {
                let index = index.cast::<usize>();
                self.stage[index] = Some(block);
                blocks.push((block, index));
            }
        }
        let mut events = vec![Event::PutBlocks(blocks)];
        events.append(&mut self.remove_filled_lines());
        self.piece_state =
            PieceState::with_initial_position(self.piece_producer.next(), self.stage_size());
        events.push(self.change_piece_event());
        events.push(self.move_piece_event());
        events
    }

    fn drop_once(&mut self) -> bool {
        if self.can_transform_piece(|index| index - euclid::TypedVector2D::new(0, 1)) {
            self.piece_state.position.y -= 1;
            true
        } else {
            false
        }
    }

    pub fn initial_events(&self) -> Vec<Event> {
        vec![self.change_piece_event(), self.move_piece_event()]
    }

    pub fn update(&mut self, delta: f64) -> Vec<Event> {
        if self.wait <= delta {
            self.wait = WAIT - (delta - self.wait);
            self.drop_piece_soft()
        } else {
            self.wait -= delta;
            vec![]
        }
    }

    fn can_move(&self, offset: isize) -> bool {
        self.can_transform_piece(|index| BlockIndexOffset::new(index.x + offset, index.y))
    }

    fn try_move(&mut self, offset: isize) -> Vec<Event> {
        if self.can_move(offset) {
            self.piece_state.position.x += offset;
            vec![self.move_piece_event()]
        } else {
            vec![]
        }
    }

    pub fn move_piece_left(&mut self) -> Vec<Event> {
        self.try_move(-1)
    }

    pub fn move_piece_right(&mut self) -> Vec<Event> {
        self.try_move(1)
    }

    pub fn drop_piece_soft(&mut self) -> Vec<Event> {
        if self.drop_once() {
            vec![self.move_piece_event()]
        } else {
            self.fix_piece()
        }
    }

    pub fn drop_piece_hard(&mut self) -> Vec<Event> {
        while self.drop_once() {}
        self.fix_piece()
    }

    fn try_change_piece(&mut self, new_piece: Piece) -> Vec<Event> {
        let new_state = PieceState::new(new_piece, self.piece_state.position);
        if new_state.blocks().all(|(index, _)| self.can_put_to(index)) {
            self.piece_state = new_state;
            vec![Event::ChangePiece(self.piece_state.piece.clone())]
        } else {
            vec![]
        }
    }

    pub fn rotate_piece_right(&mut self) -> Vec<Event> {
        self.try_change_piece(self.piece_state.piece.rotate_right())
    }

    pub fn rotate_piece_left(&mut self) -> Vec<Event> {
        self.try_change_piece(self.piece_state.piece.rotate_left())
    }
}