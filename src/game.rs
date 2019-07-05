pub mod block;
pub mod event;
pub mod piece;
mod piece_producer;

pub use block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace};
pub use event::Event;
pub use piece::Piece;

use euclid;
use euclid_ext::Points;
use piece_producer::PieceProducer;

const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const WAIT: f64 = 0.2;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PieceState {
    pub piece: Piece,
    pub position: BlockIndexOffset,
}

impl PieceState {
    pub fn new(piece: Piece, position: BlockIndexOffset) -> Self {
        Self { piece, position }
    }
}

fn initial_piece_position(piece: &Piece) -> BlockIndexOffset {
    euclid::TypedPoint2D::new(
        ((WIDTH - piece.size().width) / 2) as isize,
        -(piece.size().height as isize) / 2,
    )
}

#[derive(Debug, Clone)]
pub struct Game {
    stage: BlockGrid,
    piece_state: PieceState,
    piece_producer: PieceProducer,
    wait: f64,
}

impl Game {
    pub fn new() -> Self {
        let mut piece_producer = PieceProducer::new(piece::standards());
        let piece = piece_producer.next();
        let piece_position = initial_piece_position(&piece);
        Self {
            stage: BlockGrid::new(BlockGridSize::new(WIDTH, HEIGHT), None),
            piece_state: PieceState::new(piece, piece_position),
            piece_producer,
            wait: WAIT,
        }
    }

    pub fn stage_size(&self) -> BlockGridSize {
        self.stage.size()
    }

    pub fn block(&self, index: BlockIndex) -> Option<Block> {
        self.stage.get(index).and_then(|x| *x)
    }

    fn drop_once(&mut self) {
        let offset = self.piece_state.position;
        for index in euclid::TypedRect::from_size(self.piece_state.piece.size()).points() {
            //            let position: BlockIndexOffset =
            //                (index, offset).map(|(i, o)| i.cast::<isize>() + o.get());
            //if euclid::TypedRect::from_size(self.stage_size()).contains(&position) {

            //            }
        }
        self.piece_state.position.y += 1;
    }

    pub fn initial_events(&self) -> Vec<Event> {
        vec![
            Event::ChangePiece(&self.piece_state.piece),
            Event::MovePiece(self.piece_state.position),
        ]
    }

    pub fn update(&mut self, delta: f64) -> Vec<Event> {
        if self.wait <= delta {
            self.wait = WAIT - (delta - self.wait);
            self.drop_once();
            vec![Event::MovePiece(self.piece_state.position)]
        } else {
            self.wait -= delta;
            vec![]
        }
    }
}