pub mod block;
pub mod event;
pub mod piece;
mod piece_generator;
pub mod stage;

pub use block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace};
pub use event::{GameEvent, MoveResult, PutResult, RemoveResult};
pub use piece::Piece;
pub use stage::Stage;

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
    stage: Stage,
    piece_state: PieceState,
    piece_generator: PieceGenerator,
    wait: f64,
}

impl Game {
    pub fn new() -> Self {
        let mut piece_generator = PieceGenerator::new(piece::standards());
        let stage_size = BlockGridSize::new(WIDTH, HEIGHT);
        Self {
            stage: Stage::new(stage_size),
            piece_state: PieceState::with_initial_position(piece_generator.next(), stage_size),
            piece_generator,
            wait: WAIT,
        }
    }

    pub fn stage_size(&self) -> BlockGridSize {
        self.stage.size()
    }

    fn can_transform_piece(
        &self,
        mut transform: impl FnMut(BlockIndexOffset) -> BlockIndexOffset,
    ) -> bool {
        self.piece_state
            .blocks()
            .map(move |(index, _)| transform(index))
            .all(|index| self.stage.can_put_to(index))
    }

    fn can_move_piece(&self, offset: euclid::TypedVector2D<isize, BlockSpace>) -> bool {
        self.can_transform_piece(|index| index + offset)
    }

    fn search_hard_drop_position(&self) -> BlockIndexOffset {
        let mut offset = euclid::TypedVector2D::new(0, 0);
        while self.can_move_piece(offset + euclid::TypedVector2D::new(0, -1)) {
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

    fn piece_generation_events(&mut self) -> Vec<GameEvent> {
        vec![
            self.change_piece_event(),
            self.move_piece_event(),
            GameEvent::UpdateNextPieces(self.piece_generator.peek(3)),
        ]
    }

    fn put_piece_blocks(&mut self) -> GameEvent {
        let mut put_results = vec![];
        for (index, block) in self.piece_state.blocks() {
            if self.stage.can_put_to(index) {
                put_results.push(self.stage.put_block(index.cast::<usize>(), block));
            }
        }
        GameEvent::PutBlocks(put_results)
    }

    fn remove_filled_lines(&mut self) -> Vec<GameEvent> {
        if let Some((remove_results, move_results)) = self.stage.remove_filled_lines() {
            vec![
                GameEvent::RemoveBlocks(remove_results),
                GameEvent::MoveBlocks(move_results),
            ]
        } else {
            vec![]
        }
    }

    fn fix_piece(&mut self) -> Vec<GameEvent> {
        let mut events = vec![GameEvent::RemovePiece];
        events.push(self.put_piece_blocks());
        events.append(&mut self.remove_filled_lines());
        self.piece_state =
            PieceState::with_initial_position(self.piece_generator.next(), self.stage_size());
        events.append(&mut self.piece_generation_events());
        events
    }

    pub fn initial_events(&mut self) -> Vec<GameEvent> {
        self.piece_generation_events()
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

    fn try_move_piece(&mut self, offset: isize) -> Vec<GameEvent> {
        if self.can_move_piece(euclid::TypedVector2D::new(offset, 0)) {
            self.piece_state.position.x += offset;
            vec![self.move_piece_event()]
        } else {
            vec![]
        }
    }

    pub fn move_piece_left(&mut self) -> Vec<GameEvent> {
        self.try_move_piece(-1)
    }

    pub fn move_piece_right(&mut self) -> Vec<GameEvent> {
        self.try_move_piece(1)
    }

    pub fn drop_piece_soft(&mut self) -> Vec<GameEvent> {
        if self.can_move_piece(euclid::TypedVector2D::new(0, -1)) {
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
        if new_state
            .blocks()
            .all(|(index, _)| self.stage.can_put_to(index))
        {
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