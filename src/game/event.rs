use super::block::{Block, BlockIndexOffset, BlockIndex};
use super::piece::Piece;

pub enum Event<'a> {
    ChangePiece(&'a Piece),
    MovePiece(BlockIndexOffset),
    SetBlock(Block, BlockIndex)
}