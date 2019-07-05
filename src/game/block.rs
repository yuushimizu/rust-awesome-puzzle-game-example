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

pub type BlockGrid<U> = array2d::Array2D<Option<Block>, U>;

pub type BlockGridSize<U> = array2d::Size<U>;

pub type BlockIndex<U> = array2d::Index<U>;
