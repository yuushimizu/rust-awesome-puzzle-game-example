use super::block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockIndexOffset};
use super::event::{MoveResult, PutResult, RemoveResult};

#[derive(Debug, Clone)]
pub struct Stage {
    blocks: BlockGrid,
}

impl Stage {
    pub fn new(size: BlockGridSize) -> Self {
        Self {
            blocks: BlockGrid::new(size, None),
        }
    }

    pub fn size(&self) -> BlockGridSize {
        self.blocks.size()
    }

    pub fn can_put_to(&self, index: BlockIndexOffset) -> bool {
        index.x >= 0
            && (index.x as usize) < self.size().width
            && index.y >= 0
            && (index.y as usize >= self.size().height
                || self.blocks[index.cast::<usize>()].is_none())
    }

    pub fn put_block(&mut self, index: BlockIndex, block: Block) -> PutResult {
        self.blocks[index] = Some(block);
        PutResult {
            block: block,
            index: index,
        }
    }

    pub fn is_filled_line(&self, y: usize) -> bool {
        self.blocks
            .line(y)
            .unwrap()
            .iter()
            .all(|block| block.is_some())
    }

    pub fn filled_line_indices(&self) -> Vec<usize> {
        (0..self.size().height)
            .filter(|&y| self.is_filled_line(y))
            .collect::<Vec<_>>()
    }

    pub fn remove_block(&mut self, index: BlockIndex) -> Option<RemoveResult> {
        std::mem::replace(&mut self.blocks[index], None).map(|block| RemoveResult { block, index })
    }

    pub fn remove_line(&mut self, y: usize) -> Vec<RemoveResult> {
        let mut results = vec![];
        for index in (0..self.size().width).map(|x| BlockIndex::new(x, y)) {
            if let Some(result) = self.remove_block(index) {
                results.push(result)
            }
        }
        results
    }

    pub fn remove_lines(&mut self, indices: &[usize]) -> Vec<RemoveResult> {
        let mut results = vec![];
        for &y in indices {
            results.append(&mut self.remove_line(y));
        }
        results
    }

    pub fn move_block(
        &mut self,
        source: BlockIndex,
        destination: BlockIndex,
    ) -> Option<MoveResult> {
        std::mem::replace(&mut self.blocks[source], None).map(|block| {
            self.blocks[destination] = Some(block);
            MoveResult {
                block,
                source,
                destination,
            }
        })
    }

    pub fn move_line(&mut self, source: usize, destination: usize) -> Vec<MoveResult> {
        let mut results = vec![];
        for x in 0..self.size().width {
            if let Some(result) =
                self.move_block(BlockIndex::new(x, source), BlockIndex::new(x, destination))
            {
                results.push(result);
            }
        }
        results
    }

    pub fn remove_filled_lines(&mut self) -> Option<(Vec<RemoveResult>, Vec<MoveResult>)> {
        let line_indices = self.filled_line_indices();
        let remove_results = self.remove_lines(&line_indices);
        if remove_results.is_empty() {
            return None;
        }
        let mut line_boundaries = line_indices;
        line_boundaries.push(self.size().height);
        let mut move_results = vec![];
        for (source, destination) in line_boundaries
            .windows(2)
            .map(|area| area[0] + 1..area[1])
            .enumerate()
            .flat_map(|(index, range)| range.map(move |y| (y, y - index - 1)))
        {
            move_results.append(&mut self.move_line(source, destination));
        }
        Some((remove_results, move_results))
    }
}