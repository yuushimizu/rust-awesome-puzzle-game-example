use super::piece::Piece;
use rand::seq::SliceRandom;
use std::collections;

#[derive(Debug, Clone)]
pub struct PieceGenerator {
    source: Vec<Piece>,
    current_index: usize,
    stocks: collections::VecDeque<Piece>,
}

impl PieceGenerator {
    pub fn new(source: Vec<Piece>) -> Self {
        Self {
            current_index: source.len(),
            source,
            stocks: Default::default(),
        }
    }

    fn generate(&mut self) {
        if self.current_index >= self.source.len() {
            self.current_index = 0;
            self.source.shuffle(&mut rand::thread_rng());
        }
        self.stocks
            .push_back(self.source[self.current_index].clone());
        self.current_index += 1;
    }

    pub fn next(&mut self) -> Piece {
        self.generate();
        self.stocks.pop_front().unwrap()
    }

    pub fn peek(&mut self, count: usize) -> Vec<Piece> {
        for _ in self.stocks.len()..count {
            self.generate();
        }
        self.stocks.clone().into()
    }
}
