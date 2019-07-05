pub mod block;
pub mod piece;
pub mod piece_producer;

pub use block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockSpace};
pub use piece::Piece;

use euclid;
use piece_producer::PieceProducer;

const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const WAIT: f64 = 1.0;

type PiecePosition = euclid::TypedPoint2D<isize, BlockSpace>;

fn initial_piece_position(piece: &Piece) -> PiecePosition {
    euclid::TypedPoint2D::new(
        ((WIDTH - piece.size().width) / 2) as isize,
        -(piece.size().height as isize) / 2,
    )
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct PieceState {
    piece: Piece,
    position: PiecePosition,
}

impl PieceState {
    pub fn new(piece: Piece) -> Self {
        Self {
            position: initial_piece_position(&piece),
            piece,
        }
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
        Self {
            stage: BlockGrid::new(BlockGridSize::new(WIDTH, HEIGHT), None),
            piece_state: PieceState::new(piece_producer.next()),
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

    pub fn piece(&self) -> &Piece {
        &self.piece_state.piece
    }

    pub fn piece_position(&self) -> PiecePosition {
        self.piece_state.position
    }

    pub fn update(&mut self, delta: f64) {
        if self.wait <= delta {
            self.wait = WAIT - (delta - self.wait);
            self.piece_state.position.y += 1;
        } else {
            self.wait -= delta;
        }
    }
}