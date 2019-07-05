use super::block::BlockIndexOffset;
use super::piece::Piece;

pub enum Event<'a> {
    ChangePiece(&'a Piece),
    MovePiece(BlockIndexOffset),
}