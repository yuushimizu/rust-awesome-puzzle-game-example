use super::block::{Block, BlockIndex, BlockIndexOffset};
use super::piece::Piece;

pub enum Event {
    ChangePiece(Piece),
    MovePiece(Piece, BlockIndexOffset),
    PutBlocks(Vec<(Block, BlockIndex)>),
    RemoveBlock(Block, BlockIndex),
    MoveBlock {
        block: Block,
        source: BlockIndex,
        destination: BlockIndex,
    },
}