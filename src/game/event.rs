use super::block::{Block, BlockIndex, BlockIndexOffset};
use super::piece::Piece;

pub enum Event {
    ChangePiece(Piece),
    MovePiece(Piece, BlockIndexOffset),
    PutBlocks(Vec<(Block, BlockIndex)>),
    RemoveBlocks(Vec<(Block, BlockIndex)>),
    MoveBlocks(Vec<(Block, BlockIndex, BlockIndex)>)
}