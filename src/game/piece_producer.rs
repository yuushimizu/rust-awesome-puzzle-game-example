use super::piece::Piece;
use rand;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct PieceProducer {
    source: Vec<Piece>,
    current_index: usize,
}

impl PieceProducer {
    pub fn new(mut source: Vec<Piece>) -> Self {
        source.shuffle(&mut rand::thread_rng());
        Self {
            source,
            current_index: 0,
        }
    }

    pub fn next(&mut self) -> Piece {
        if self.current_index >= self.source.len() {
            self.current_index = 0;
            self.source.shuffle(&mut rand::thread_rng());
        }
        let result = self.source[self.current_index].clone();
        self.current_index += 1;
        result
    }
}
