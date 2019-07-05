use super::block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockNumber};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceSpace {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Piece {
    blocks: BlockGrid<PieceSpace>,
}

impl Piece {
    pub fn new(blocks: BlockGrid<PieceSpace>) -> Self {
        Self { blocks }
    }

    pub fn size(&self) -> BlockGridSize<PieceSpace> {
        self.blocks.size()
    }

    pub fn blocks(&self) -> &BlockGrid<PieceSpace> {
        &self.blocks
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
