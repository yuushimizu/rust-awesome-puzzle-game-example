use super::block::{Block, BlockIndex, BlockIndexOffset};
use super::piece::Piece;

pub enum Event<'a> {
    ChangePiece(&'a Piece),
    MovePiece(BlockIndexOffset),
    SetBlock(Block, BlockIndex),
}