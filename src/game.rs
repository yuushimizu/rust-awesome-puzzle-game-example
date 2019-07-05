pub mod block;
pub mod piece;
pub mod piece_producer;

pub use block::{Block, BlockGrid, BlockGridSize, BlockPosition, BlockSpace};
pub use piece::Piece;

use piece_producer::PieceProducer;

const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const WAIT: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct Game {
    stage: BlockGrid,
    piece: Piece,
    piece_producer: PieceProducer,
    wait: f64,
}

impl Game {
    pub fn new() -> Self {
        let mut piece_producer = PieceProducer::new(piece::standards());
        Self {
            stage: BlockGrid::new(BlockGridSize::new(WIDTH, HEIGHT), None),
            piece: piece_producer.next(),
            piece_producer,
            wait: WAIT,
        }
    }

    pub fn stage_size(&self) -> BlockGridSize {
        self.stage.size()
    }

    pub fn block(&self, position: BlockPosition) -> Option<Block> {
        self.stage.get(position).and_then(|x| *x)
    }

    pub fn piece(&self) -> &Piece {
        &self.piece
    }

    pub fn update(&mut self, delta: f64) {
        if self.wait <= delta {
            self.wait = WAIT - (delta - self.wait);
        } else {
            self.wait -= delta;
        }
    }
}