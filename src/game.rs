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
const WAIT: f64 = 0.8;

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

    fn can_put_to(&self, index: BlockIndexOffset) -> bool {
        index.x >= 0
            && (index.y < 0 || {
                let index = index.cast::<usize>();
                index.x < self.stage_size().width
                    && index.y < self.stage_size().height
                    && self.stage[index].is_none()
            })
    }

    fn fix_piece(&mut self) -> Vec<Event> {
        let mut events = vec![];
        for (index, block) in self.piece_state.piece_blocks() {
            debug_assert!(self.can_put_to(index));
            if index.y >= 0 {
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

    fn drop_once(&mut self) -> bool {
        if self
            .piece_state
            .piece_blocks()
            .map(|(index, _)| index + euclid::TypedVector2D::new(0, 1))
            .any(|bottom| !self.can_put_to(bottom))
        {
            return false;
        }
        self.piece_state.position.y += 1;
        true
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
            self.drop_piece_soft()
        } else {
            self.wait -= delta;
            vec![]
        }
    }

    fn can_move(&self, offset: isize) -> bool {
        !self
            .piece_state
            .piece_blocks()
            .map(|(index, _)| BlockIndexOffset::new(index.x + offset, index.y))
            .any(|index| !self.can_put_to(index))
    }

    fn try_move(&mut self, offset: isize) -> Vec<Event> {
        if self.can_move(offset) {
            self.piece_state.position.x += offset;
            vec![Event::MovePiece(self.piece_state.position)]
        } else {
            vec![]
        }
    }

    pub fn move_piece_left(&mut self) -> Vec<Event> {
        self.try_move(-1)
    }

    pub fn move_piece_right(&mut self) -> Vec<Event> {
        self.try_move(1)
    }

    pub fn drop_piece_soft(&mut self) -> Vec<Event> {
        if self.drop_once() {
            vec![Event::MovePiece(self.piece_state.position)]
        } else {
            self.fix_piece()
        }
    }

    pub fn drop_piece_hard(&mut self) -> Vec<Event> {
        while self.drop_once() {}
        self.fix_piece()
    }

    pub fn rotate_piece_right(&mut self) -> Vec<Event> {
        vec![]
    }

    pub fn rotate_piece_left(&mut self) -> Vec<Event> {
        vec![]
    }
}