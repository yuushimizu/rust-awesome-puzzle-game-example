use crate::assets::{BlockFace, Texture};
use crate::game::{
    Block, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace, Game, GameEvent, Piece,
};
use crate::scene_context::SceneContext;
use crate::sprite_ext::{AddTo, MoveTo, MovedTo, PixelPosition, RemoveAllChildren, Sprite};
use array2d;
use euclid;
use piston_window::*;
use std::collections;
use uuid;

const TILE_SIZE: f64 = 8.0;

const SCALE: f64 = 2.0;

trait ToPixelSpace {
    type Output;

    fn to_pixel_space(self, grid_size: BlockGridSize) -> Self::Output;
}

impl ToPixelSpace for BlockIndexOffset {
    type Output = PixelPosition;

    fn to_pixel_space(self, grid_size: BlockGridSize) -> PixelPosition {
        PixelPosition::new(
            TILE_SIZE / 2.0 + self.x as f64 * TILE_SIZE,
            TILE_SIZE / 2.0 + ((grid_size.height as isize) - 1 - self.y) as f64 * TILE_SIZE,
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
    GameEvent(GameEvent),
    Run(Box<dyn FnOnce(&mut GameScene, &mut SceneContext)>),
}

pub struct GameScene {
    game: Game,
    scene: sprite::Scene<Texture>,
    stage_id: uuid::Uuid,
    piece_id: uuid::Uuid,
    block_ids: array2d::Array2D<Option<uuid::Uuid>, BlockSpace>,
    jobs: collections::VecDeque<Job>,
}

impl GameScene {
    pub fn new(context: &mut SceneContext) -> Self {
        let game = Game::new();
        let mut stage = context
            .empty_sprite()
            .moved_to(PixelPosition::new(TILE_SIZE * 10.0, 0.0));
        stage.set_scale(SCALE, SCALE);
        use euclid_ext::Points;
        for index in euclid::TypedRect::from_size(game.stage_size()).points() {
            Sprite::from_texture(context.assets.background_tile_texture())
                .moved_to(index.to_pixel_space(game.stage_size()))
                .add_to(&mut stage);
        }
        let initial_events = game.initial_events();
        let mut scene = sprite::Scene::new();
        let mut result = Self {
            piece_id: context.empty_sprite().add_to(&mut stage),
            stage_id: scene.add_child(stage),
            block_ids: array2d::Array2D::new(game.stage_size(), None),
            game,
            scene,
            jobs: Default::default(),
        };
        result.apply_game_events(initial_events, context);
        result
    }

    fn stage_size(&self) -> BlockGridSize {
        self.game.stage_size()
    }

    fn stage_sprite(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.stage_id).unwrap()
    }

    fn piece_sprite(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.piece_id).unwrap()
    }

    fn is_ready(&self) -> bool {
        use euclid_ext::Points;
        for index in euclid::TypedRect::from_size(self.stage_size()).points() {
            if let Some(id) = self.block_ids[index] {
                if self.scene.running_for_child(id).unwrap_or(0) > 0 {
                    return false;
                }
            }
        }
        true
    }

    fn change_piece(&mut self, piece: Piece, context: &mut SceneContext) {
        self.piece_sprite().remove_all_children();
        for (index, block) in piece.blocks() {
            sprite::Sprite::from_texture(context.assets.block_texture(block, BlockFace::Sleep))
                .moved_to(index.to_pixel_space(piece.size()))
                .add_to(self.piece_sprite());
        }
    }

    fn move_piece(
        &mut self,
        piece: Piece,
        position: BlockIndexOffset,
        _context: &mut SceneContext,
    ) {
        let position = position.to_pixel_space(self.stage_size())
            - BlockIndex::new(0, 0)
                .to_pixel_space(piece.size())
                .to_vector();
        self.piece_sprite().move_to(position);
    }

    fn remove_piece(&mut self, _context: &mut SceneContext) {
        self.piece_sprite().remove_all_children();
    }

    fn put_blocks(&mut self, blocks: Vec<(Block, BlockIndex)>, context: &mut SceneContext) {
        for (block, index) in blocks {
            if let Some(old_id) = self.block_ids[index] {
                self.stage_sprite().remove_child(old_id);
            }
            self.block_ids[index] = Some(
                sprite::Sprite::from_texture(
                    context.assets.block_texture(block, BlockFace::Normal),
                )
                .moved_to(index.to_pixel_space(self.stage_size()))
                .add_to(self.stage_sprite()),
            );
        }
    }

    fn add_removing_action(&mut self, block: Block, index: BlockIndex, context: &mut SceneContext) {
        use ai_behavior::{Action, Sequence};
        use sprite::{Ease, EaseFunction, ScaleTo};
        let id = self.block_ids[index].unwrap();
        let texture = context.assets.block_texture(block, BlockFace::Happy);
        self.scene.child_mut(id).unwrap().set_texture(texture);
        self.scene.run(
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
            .push_front(Job::Run(Box::new(move |this, _context| {
                for (_block, index) in blocks {
                    let id = std::mem::replace(&mut this.block_ids[index], None).unwrap();
                    this.stage_sprite().remove_child(id);
                }
            })));
    }

    fn move_blocks(
        &mut self,
        moves: Vec<(Block, BlockIndex, BlockIndex)>,
        _context: &mut SceneContext,
    ) {
        for (_block, source, destination) in moves {
            let id = std::mem::replace(&mut self.block_ids[source], None).unwrap();
            let position = destination.to_pixel_space(self.stage_size());
            self.scene.child_mut(id).unwrap().move_to(position);
            self.block_ids[destination] = Some(id);
        }
    }

    fn apply_game_event(&mut self, event: GameEvent, context: &mut SceneContext) {
        use GameEvent::*;
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
            if !self.is_ready() {
                break;
            }
            if let Some(job) = self.jobs.pop_front() {
                match job {
                    Job::GameEvent(event) => {
                        self.apply_game_event(event, context);
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

    fn apply_game_events(&mut self, events: Vec<GameEvent>, context: &mut SceneContext) {
        self.jobs
            .extend(events.into_iter().map(|event| Job::GameEvent(event)));
        self.execute_jobs(context);
    }

    fn update(&mut self, delta: f64, context: &mut SceneContext) {
        self.execute_jobs(context);
        if self.is_ready() {
            let events = self.game.update(delta);
            self.apply_game_events(events, context);
        }
    }

    fn input(&mut self, input: Input, context: &mut SceneContext) {
        if !self.is_ready() {
            return;
        }
        match input {
            Input::Button(ButtonArgs {
                state: ButtonState::Press,
                button: Button::Keyboard(key),
                ..
            }) => {
                let events = match key {
                    Key::Left => self.game.move_piece_left(),
                    Key::Right => self.game.move_piece_right(),
                    Key::Down => self.game.drop_piece_soft(),
                    Key::Up => self.game.drop_piece_hard(),
                    Key::Z => self.game.rotate_piece_left(),
                    Key::X => self.game.rotate_piece_right(),
                    _ => return,
                };
                self.apply_game_events(events, context);
            }
            _ => {}
        }
    }

    pub fn apply_window_event(
        &mut self,
        event: Event,
        window: &mut PistonWindow,
        scene_context: &mut SceneContext,
    ) {
        self.scene.event(&event);
        match event {
            Event::Loop(Loop::Update(arg)) => {
                self.update(arg.dt, scene_context);
            }
            Event::Loop(Loop::Render(_)) => {
                window.draw_2d(&event, |c, g, _| {
                    clear([0.0, 0.0, 0.0, 1.0], g);
                    self.scene.draw(c.transform, g);
                });
            }
            Event::Input(input, _) => {
                self.input(input, scene_context);
            }
            _ => {}
        }
    }
}