use super::block::{Block, BlockIndex, BlockIndexOffset};
use super::piece::Piece;

pub enum GameEvent {
    ChangePiece {piece: Piece, guide_position: BlockIndexOffset},
    MovePiece {piece: Piece, position: BlockIndexOffset, guide_position: BlockIndexOffset},
    RemovePiece,
    UpdateNextPieces(Vec<Piece>),
    PutBlocks(Vec<(Block, BlockIndex)>),
    RemoveBlocks(Vec<(Block, BlockIndex)>),
    MoveBlocks(Vec<(Block, BlockIndex, BlockIndex)>),
}