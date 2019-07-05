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
}

pub struct PieceBlocks<'a, I: iter::Iterator<Item = BlockIndex>> {
    pub piece: &'a Piece,
    indices: I,
}

impl<'a, I: iter::Iterator<Item = BlockIndex>> iter::Iterator for PieceBlocks<'a, I> {
    type Item = (BlockIndex, Block);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(index) = self.indices.next() {
            if let Some(block) = self.piece.block(index) {
                return Some((index, block));
            }
        }
        None
    }
}

impl Piece {
    pub fn blocks(&self) -> PieceBlocks<impl iter::Iterator<Item = BlockIndex>> {
        use euclid_ext::Points;
        PieceBlocks {
            indices: euclid::TypedRect::from_size(self.size()).points(),
            piece: self,
        }
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
        (4, [(0, 1), (1, 1), (2, 1), (3, 1)]), // I
        (2, [(0, 0), (1, 0), (0, 1), (1, 1)]), // O
        (3, [(1, 0), (2, 0), (0, 1), (1, 1)]), // S
        (3, [(0, 0), (1, 0), (1, 1), (2, 1)]), // Z
        (3, [(0, 0), (0, 1), (1, 1), (2, 1)]), // J
        (3, [(2, 0), (0, 1), (1, 1), (2, 1)]), // L
        (3, [(1, 0), (0, 1), (1, 1), (2, 1)]), // T
    ]
    .iter()
    .enumerate()
    .map(|(number, (size, blocks))| piece(*size, number, blocks))
    .collect()
}
