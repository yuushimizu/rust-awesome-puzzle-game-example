use array2d;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Block {
    pub number: u32,
}

impl Block {
    pub fn new(number: u32) -> Self {
        Self { number }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BlockSpace {}

pub type BlockGrid = array2d::Array2D<Block, BlockSpace>;

pub type BlockPosition = array2d::Index<BlockSpace>;