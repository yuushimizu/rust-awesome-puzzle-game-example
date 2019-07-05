use array2d;

pub type BlockNumber = u32;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Block {
    pub number: BlockNumber,
}

impl Block {
    pub fn new(number: BlockNumber) -> Self {
        Self { number }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BlockSpace {}

pub type BlockGrid = array2d::Array2D<Option<Block>, BlockSpace>;

pub type BlockGridSize = array2d::Size<BlockSpace>;

pub type BlockIndex = array2d::Index<BlockSpace>;
