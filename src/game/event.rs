use super::piece::{Piece, PiecePosition};

pub enum Event<'a> {
    ChangePiece(&'a Piece),
    MovePiece(PiecePosition),
}