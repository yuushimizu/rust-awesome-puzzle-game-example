use crate::game::block::{BlockGrid};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Piece {
    blocks: BlockGrid,
}
