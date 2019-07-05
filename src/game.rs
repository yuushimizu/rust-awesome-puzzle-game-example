pub mod block;
pub mod event;
pub mod piece;
mod piece_producer;

pub use block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace};
pub use event::Event;
pub use piece::Piece;

use euclid;
use euclid_ext::{Map2D, Points};
use piece_producer::PieceProducer;

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
            -(piece.size().height as isize) / 2,
        );
        Self::new(piece, position)
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

    pub fn block(&self, index: BlockIndex) -> Option<Block> {
        self.stage.get(index).and_then(|x| *x)
    }

    fn fix_piece(&mut self) -> Vec<Event> {
        // fix
        self.piece_state =
            PieceState::with_initial_position(self.piece_producer.next(), self.stage_size());
        vec![
            Event::ChangePiece(&self.piece_state.piece),
            Event::MovePiece(self.piece_state.position),
        ]
    }

    fn drop_once(&mut self) -> Vec<Event> {
        let offset = self.piece_state.position;
        for index in euclid::TypedRect::from_size(self.piece_state.piece.size()).points() {
            if self.piece_state.piece.blocks()[index].is_none() {
                continue;
            }
            let position: BlockIndexOffset = (offset, index.cast::<isize>()).map(|(o, i)| o + i);
            debug_assert!(position.x >= 0 || (position.x as usize) < self.stage_size().width);
            let bottom = position + euclid::TypedVector2D::new(0, 1);
            if bottom.y < 0 {
                continue;
            }
            let bottom = bottom.cast::<usize>();
            if bottom.y >= self.stage_size().height || self.stage[bottom].is_some() {
                return self.fix_piece();
            }
        }
        self.piece_state.position.y += 1;
        vec![Event::MovePiece(self.piece_state.position)]
    }

    pub fn initial_events(&self) -> Vec<Event> {
        vec![
            Event::ChangePiece(&self.piece_state.piece),
            Event::MovePiece(self.piece_state.position),
        ]
    }

    pub fn update(&mut self, delta: f64) -> Vec<Event> {
        if self.wait <= delta {
            self.wait = WAIT - (delta - self.wait);
            self.drop_once()
        } else {
            self.wait -= delta;
            vec![]
        }
    }
}