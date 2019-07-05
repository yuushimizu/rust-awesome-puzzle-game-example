use super::piece::PieceState;

pub enum Event<'a> {
    ChangePiece(&'a PieceState),
}