pub mod block;
pub mod piece;
pub mod piece_producer;

pub use block::{Block, BlockGrid, BlockGridSize, BlockPosition, BlockSpace};
pub use piece::Piece;

use piece_producer::PieceProducer;

const WIDTH: usize = 10;
const HEIGHT: usize = 20;

#[derive(Debug, Clone)]
pub struct Game {
    stage: BlockGrid,
    piece_producer: PieceProducer,
}

impl Game {
    pub fn new() -> Self {
        Self {
            stage: BlockGrid::new(BlockGridSize::new(WIDTH, HEIGHT), None),
            piece_producer: PieceProducer::new(piece::standards()),
        }
    }

    pub fn stage_size(&self) -> BlockGridSize {
        self.stage.size()
    }

    pub fn block(&self, position: BlockPosition) -> Option<Block> {
        self.stage.get(position).and_then(|x| *x)
    }
}