use super::block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockNumber};
use euclid;
use std::iter;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Piece {
    blocks: BlockGrid,
}

impl Piece {
    pub fn new(blocks: BlockGrid) -> Self {
        Self { blocks }
    }

    pub fn size(&self) -> BlockGridSize {
        self.blocks.size()
    }

    pub fn block(&self, index: BlockIndex) -> Option<Block> {
        self.blocks.get(index).and_then(|x| *x)
    }

    pub fn blocks<'a>(&'a self) -> impl iter::Iterator<Item = (BlockIndex, Block)> + 'a {
        use euclid_ext::Points;
        euclid::TypedRect::from_size(self.size())
            .points()
            .filter_map(move |index| self.block(index).map(|block| (index, block)))
    }

    fn transform(&self, mut transform: impl FnMut(BlockIndex) -> BlockIndex) -> Self {
        let mut blocks = BlockGrid::new(self.size(), None);
        for (index, block) in self.blocks() {
            blocks[transform(index)] = Some(block);
        }
        Self::new(blocks)
    }

    pub fn rotate_left(&self) -> Self {
        self.transform(|index| BlockIndex::new(index.y, self.size().height - 1 - index.x))
    }

    pub fn rotate_right(&self) -> Self {
        self.transform(|index| BlockIndex::new(self.size().width - 1 - index.y, index.x))
    }
}

fn piece(size: usize, number: usize, blocks: &[(usize, usize)]) -> Piece {
    let mut grid = BlockGrid::new(BlockGridSize::new(size, size), None);
    for (x, y) in blocks {
        grid[BlockIndex::new(*x, *y)] = Some(Block::new(number as BlockNumber))
    }
    Piece::new(grid)
}

pub fn standards() -> Vec<Piece> {
    [
        (4, [(0, 2), (1, 2), (2, 2), (3, 2)]), // I
        (2, [(0, 0), (1, 0), (0, 1), (1, 1)]), // O
        (3, [(1, 2), (2, 2), (0, 1), (1, 1)]), // S
        (3, [(0, 2), (1, 2), (1, 1), (2, 1)]), // Z
        (3, [(0, 2), (0, 1), (1, 1), (2, 1)]), // J
        (3, [(2, 2), (0, 1), (1, 1), (2, 1)]), // L
        (3, [(1, 2), (0, 1), (1, 1), (2, 1)]), // T
    ]
    .iter()
    .enumerate()
    .map(|(number, (size, blocks))| piece(*size, number, blocks))
    .collect()
}
