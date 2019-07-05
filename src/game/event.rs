use super::piece::Piece;
use super::PiecePosition;

pub enum Event<'a> {
    ChangePiece(&'a Piece),
    MovePiece(PiecePosition),
}