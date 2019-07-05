use super::block::{Block, BlockIndex, BlockIndexOffset};
use super::piece::Piece;

pub enum GameEvent {
    ChangePiece(Piece),
    MovePiece(Piece, BlockIndexOffset),
    RemovePiece,
    UpdateNextPieces(Vec<Piece>),
    PutBlocks(Vec<(Block, BlockIndex)>),
    RemoveBlocks(Vec<(Block, BlockIndex)>),
    MoveBlocks(Vec<(Block, BlockIndex, BlockIndex)>),
}