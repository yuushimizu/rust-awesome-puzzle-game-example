pub mod block;
pub mod event;
pub mod piece;
mod piece_producer;

pub use block::{Block, BlockGrid, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace};
pub use event::Event;
pub use piece::Piece;

use euclid;

use piece_producer::PieceProducer;
use std::iter;
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

    pub fn with_initial_position(piece: Piece, stage_size: BlockGridSize) -> Self {
        let position = BlockIndexOffset::new(
            ((stage_size.width - piece.size().width) / 2) as isize,
            -(piece.size().height as isize) / 2,
        );
        Self::new(piece, position)
    }
}

struct StagePieceBlocks<'a, I: iter::Iterator<Item = BlockIndex>> {
    blocks: piece::PieceBlocks<'a, I>,
    piece_position: BlockIndexOffset,
}

impl<'a, I: iter::Iterator<Item = BlockIndex>> iter::Iterator for StagePieceBlocks<'a, I> {
    type Item = (BlockIndexOffset, Block);

    fn next(&mut self) -> Option<Self::Item> {
        use euclid_ext::Map2D;
        self.blocks.next().map(|(index, block)| {
            (
                (self.piece_position, index.cast::<isize>()).map(|(i, p)| i + p),
                block,
            )
        })
    }
}

impl PieceState {
    fn piece_blocks(&self) -> StagePieceBlocks<impl iter::Iterator<Item = BlockIndex>> {
        StagePieceBlocks {
            blocks: self.piece.blocks(),
            piece_position: self.position,
        }
    }
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
        let stage_size = BlockGridSize::new(WIDTH, HEIGHT);
        Self {
            stage: BlockGrid::new(stage_size, None),
            piece_state: PieceState::with_initial_position(piece, stage_size),
            piece_producer,
            wait: WAIT,
        }
    }

    pub fn stage_size(&self) -> BlockGridSize {
        self.stage.size()
    }

    fn fix_piece(&mut self) -> Vec<Event> {
        let mut events = vec![];
        for (index, block) in self.piece_state.piece_blocks() {
            if index.y >= 0 && (index.y as usize) < self.stage_size().height {
                let index = index.cast::<usize>();
                self.stage[index] = Some(block);
                events.push(Event::SetBlock(block, index));
            }
        }
        self.piece_state =
            PieceState::with_initial_position(self.piece_producer.next(), self.stage_size());
        events.push(Event::ChangePiece(&self.piece_state.piece));
        events.push(Event::MovePiece(self.piece_state.position));
        events
    }

    fn drop_once(&mut self) -> Vec<Event> {
        if self
            .piece_state
            .piece_blocks()
            .map(|(index, _)| index + euclid::TypedVector2D::new(0, 1))
            .filter(|bottom| {
                bottom.y >= 0
                    && (bottom.y as usize >= self.stage_size().height
                        || self.stage[bottom.cast::<usize>()].is_some())
            })
            .next()
            .is_some()
        {
            return self.fix_piece();
        }
        self.piece_state.position.y += 1;
        vec![Event::MovePiece(self.piece_state.position)]
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
            self.drop_once()
        } else {
            self.wait -= delta;
            vec![]
        }
    }
}