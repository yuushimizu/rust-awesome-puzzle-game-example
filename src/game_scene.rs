use crate::assets::BlockFace;
use crate::game::{
    Block, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace, Event, Game, Piece,
};
use crate::scene_context::SceneContext;
use crate::sprite_ext::{AddTo, MoveTo, MovedTo, PixelPosition, RemoveAllChildren, Sprite};
use array2d;
use piston_window::*;
use std::collections;
use uuid;

const TILE_SIZE: f64 = 8.0;

trait ToPixelSpace {
    type Output;

    fn to_pixel_space(self, grid_size: BlockGridSize) -> Self::Output;
}

impl ToPixelSpace for BlockIndexOffset {
    type Output = PixelPosition;

    fn to_pixel_space(self, grid_size: BlockGridSize) -> PixelPosition {
        PixelPosition::new(
            self.x as f64 * TILE_SIZE,
            ((grid_size.height as isize) - 1 - self.y) as f64 * TILE_SIZE,
        )
    }
}

impl ToPixelSpace for BlockIndex {
    type Output = PixelPosition;

    fn to_pixel_space(self, grid_size: BlockGridSize) -> PixelPosition {
        self.cast::<isize>().to_pixel_space(grid_size)
    }
}

enum Job {
    Event(Event),
    Run(Box<dyn FnOnce(&mut GameSceneSprite, &mut SceneContext)>),
}

struct GameSceneSprite {
    stage_size: BlockGridSize,
    stage_id: uuid::Uuid,
    piece_id: uuid::Uuid,
    block_ids: array2d::Array2D<Option<uuid::Uuid>, BlockSpace>,
    jobs: collections::VecDeque<Job>,
}

impl GameSceneSprite {
    fn new(stage_size: BlockGridSize, context: &mut SceneContext) -> Self {
        let mut stage = context.empty_sprite().moved_to(PixelPosition::new(
            TILE_SIZE / 2.0 + TILE_SIZE * 5.0,
            TILE_SIZE / 2.0,
        ));
        use euclid_ext::Points;
        for index in euclid::TypedRect::from_size(stage_size).points() {
            Sprite::from_texture(context.assets.background_tile_texture())
                .moved_to(index.to_pixel_space(stage_size))
                .add_to(&mut stage);
        }
        Self {
            stage_size,
            piece_id: context.empty_sprite().add_to(&mut stage),
            stage_id: stage.add_to(context.root()),
            block_ids: array2d::Array2D::new(stage_size, None),
            jobs: Default::default(),
        }
    }

    fn stage_sprite<'a>(&self, context: &'a mut SceneContext) -> &'a mut Sprite {
        context.child_mut(self.stage_id).unwrap()
    }

    fn piece_sprite<'a>(&self, context: &'a mut SceneContext) -> &'a mut Sprite {
        context.child_mut(self.piece_id).unwrap()
    }

    fn block_action_is_running(&self, context: &SceneContext) -> bool {
        use euclid_ext::Points;
        for index in euclid::TypedRect::from_size(self.stage_size).points() {
            if let Some(id) = self.block_ids[index] {
                if context.scene.running_for_child(id).unwrap_or(0) > 0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_ready(&self, context: &SceneContext) -> bool {
        !self.block_action_is_running(context)
    }

    fn change_piece(&mut self, piece: Piece, context: &mut SceneContext) {
        self.piece_sprite(context).remove_all_children();
        for (index, block) in piece.blocks() {
            sprite::Sprite::from_texture(context.assets.block_texture(block, BlockFace::Sleep))
                .moved_to(index.to_pixel_space(piece.size()))
                .add_to(self.piece_sprite(context));
        }
    }

    fn move_piece(&mut self, piece: Piece, position: BlockIndexOffset, context: &mut SceneContext) {
        self.piece_sprite(context).move_to(
            position.to_pixel_space(self.stage_size)
                - BlockIndex::new(0, 0)
                    .to_pixel_space(piece.size())
                    .to_vector(),
        );
    }

    fn remove_piece(&mut self, context: &mut SceneContext) {
        self.piece_sprite(context).remove_all_children();
    }

    fn put_blocks(&mut self, blocks: Vec<(Block, BlockIndex)>, context: &mut SceneContext) {
        for (block, index) in blocks {
            if let Some(old_id) = self.block_ids[index] {
                self.stage_sprite(context).remove_child(old_id);
            }
            self.block_ids[index] = Some(
                sprite::Sprite::from_texture(
                    context.assets.block_texture(block, BlockFace::Normal),
                )
                .moved_to(index.to_pixel_space(self.stage_size))
                .add_to(self.stage_sprite(context)),
            );
        }
    }

    fn add_removing_action(&self, block: Block, index: BlockIndex, context: &mut SceneContext) {
        use ai_behavior::{Action, Sequence, Wait};
        use sprite::{Ease, EaseFunction, ScaleTo};
        let id = self.block_ids[index].unwrap();
        let texture = context.assets.block_texture(block, BlockFace::Happy);
        context.child_mut(id).unwrap().set_texture(texture);
        context.scene.run(
            id,
            &Sequence(vec![
                Action(Ease(
                    EaseFunction::CubicOut,
                    Box::new(ScaleTo(0.15, 0.6, 1.4)),
                )),
                Action(Ease(
                    EaseFunction::CubicOut,
                    Box::new(ScaleTo(0.15, 4.0, 0.0)),
                )),
            ]),
        );
    }

    fn remove_blocks(&mut self, blocks: Vec<(Block, BlockIndex)>, context: &mut SceneContext) {
        for &(block, index) in &blocks {
            self.add_removing_action(block, index, context);
        }
        self.jobs
            .push_front(Job::Run(Box::new(move |this, context| {
                for (_block, index) in blocks {
                    let id = std::mem::replace(&mut this.block_ids[index], None).unwrap();
                    this.stage_sprite(context).remove_child(id);
                }
            })));
    }

    fn move_blocks(
        &mut self,
        moves: Vec<(Block, BlockIndex, BlockIndex)>,
        context: &mut SceneContext,
    ) {
        for (_block, source, destination) in moves {
            let id = std::mem::replace(&mut self.block_ids[source], None).unwrap();
            context
                .child_mut(id)
                .unwrap()
                .move_to(destination.to_pixel_space(self.stage_size));
            self.block_ids[destination] = Some(id);
        }
    }

    fn apply_event(&mut self, event: Event, context: &mut SceneContext) {
        use Event::*;
        match event {
            ChangePiece(piece) => {
                self.change_piece(piece, context);
            }
            MovePiece(piece, position) => {
                self.move_piece(piece, position, context);
            }
            RemovePiece => {
                self.remove_piece(context);
            }
            PutBlocks(blocks) => {
                self.put_blocks(blocks, context);
            }
            RemoveBlocks(blocks) => {
                self.remove_blocks(blocks, context);
            }
            MoveBlocks(moves) => {
                self.move_blocks(moves, context);
            }
        }
    }

    fn execute_jobs(&mut self, context: &mut SceneContext) {
        loop {
            if self.block_action_is_running(context) {
                break;
            }
            if let Some(job) = self.jobs.pop_front() {
                match job {
                    Job::Event(event) => {
                        self.apply_event(event, context);
                    }
                    Job::Run(f) => {
                        f(self, context);
                    }
                }
            } else {
                break;
            }
        }
    }

    fn update(&mut self, context: &mut SceneContext) {
        self.execute_jobs(context);
    }

    fn apply_events(&mut self, events: Vec<Event>, context: &mut SceneContext) {
        self.jobs
            .extend(events.into_iter().map(|event| Job::Event(event)));
        self.execute_jobs(context);
    }
}

pub struct GameScene {
    game: Game,
    sprite: GameSceneSprite,
}

impl GameScene {
    pub fn new(context: &mut SceneContext) -> Self {
        let game = Game::new();
        let mut sprite = GameSceneSprite::new(game.stage_size(), context);
        sprite.apply_events(game.initial_events(), context);
        Self { game, sprite }
    }

    pub fn update(&mut self, delta: f64, context: &mut SceneContext) {
        self.sprite.update(context);
        if self.sprite.is_ready(context) {
            self.sprite.apply_events(self.game.update(delta), context);
        }
    }

    pub fn input(&mut self, input: Input, context: &mut SceneContext) {
        if !self.sprite.is_ready(context) {
            return;
        }
        match input {
            Input::Button(ButtonArgs {
                state: ButtonState::Press,
                button: Button::Keyboard(key),
                ..
            }) => {
                self.sprite.apply_events(
                    match key {
                        Key::Left => self.game.move_piece_left(),
                        Key::Right => self.game.move_piece_right(),
                        Key::Down => self.game.drop_piece_soft(),
                        Key::Up => self.game.drop_piece_hard(),
                        Key::Z => self.game.rotate_piece_left(),
                        Key::X => self.game.rotate_piece_right(),
                        _ => return,
                    },
                    context,
                );
            }
            _ => {}
        }
    }
}