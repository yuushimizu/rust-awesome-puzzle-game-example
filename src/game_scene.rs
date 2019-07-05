use crate::assets::{BlockFace, Texture};
use crate::game::{
    Block, BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace, Game, GameEvent, Piece,
};
use crate::scene_context::SceneContext;
use crate::sprite_ext::{AddTo, MoveTo, MovedTo, PixelPosition, RemoveAllChildren, Sprite};
use piston_window::*;
use std::collections;

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
    Run(Box<dyn FnOnce(&mut GameScene)>),
}

fn put_background_tile_sprites(
    sprite: &mut Sprite,
    size: BlockGridSize,
    context: &mut SceneContext,
) {
    use euclid_ext::Points;
    for index in euclid::TypedRect::from_size(size).points() {
        Sprite::from_texture(context.assets.background_tile_texture())
            .moved_to(index.to_pixel_space(size))
            .add_to(sprite);
    }
}

pub struct GameScene<'a> {
    game: Game,
    scene: sprite::Scene<Texture>,
    context: &'a mut SceneContext,
    stage_sprite_id: uuid::Uuid,
    piece_sprite_id: uuid::Uuid,
    next_pieces_sprite_id: uuid::Uuid,
    block_ids: array2d::Array2D<Option<uuid::Uuid>, BlockSpace>,
    jobs: collections::VecDeque<Job>,
}

impl<'a> GameScene<'a> {
    pub fn new(context: &'a mut SceneContext) -> Self {
        let game = Game::new();
        let mut scene = sprite::Scene::new();
        let mut root_sprite = context.empty_sprite();
        root_sprite.set_scale(SCALE, SCALE);
        let mut stage_sprite = context
            .empty_sprite()
            .moved_to(PixelPosition::new(TILE_SIZE * 3.0, 0.0));
        put_background_tile_sprites(&mut stage_sprite, game.stage_size(), context);
        let piece_sprite_id = stage_sprite.add_child(context.empty_sprite());
        let stage_sprite_id = root_sprite.add_child(stage_sprite);
        let mut next_pieces_sprite = context.empty_sprite().moved_to(PixelPosition::new(
            TILE_SIZE * (game.stage_size().width as f64 + 4.5),
            TILE_SIZE * 1.0,
        ));
        put_background_tile_sprites(
            &mut next_pieces_sprite,
            BlockGridSize::new(6, 6 * 3),
            context,
        );
        let next_pieces_sprite_id = root_sprite.add_child(next_pieces_sprite);
        scene.add_child(root_sprite);
        let initial_events = game.initial_events();
        let mut result = Self {
            block_ids: array2d::Array2D::new(game.stage_size(), None),
            game,
            scene,
            context,
            piece_sprite_id,
            stage_sprite_id,
            next_pieces_sprite_id,
            jobs: Default::default(),
        };
        result.apply_game_events(initial_events);
        result
    }

    fn stage_size(&self) -> BlockGridSize {
        self.game.stage_size()
    }

    fn stage_sprite(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.stage_sprite_id).unwrap()
    }

    fn piece_sprite(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.piece_sprite_id).unwrap()
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

    fn put_piece_sprites(context: &mut SceneContext, sprite: &mut Sprite, piece: &Piece) {
        for (index, block) in piece.blocks() {
            sprite::Sprite::from_texture(context.assets.block_texture(block, BlockFace::Sleep))
                .moved_to(index.to_pixel_space(piece.size()))
                .add_to(sprite);
        }
    }

    fn change_piece(&mut self, piece: Piece) {
        self.piece_sprite().remove_all_children();
        let piece_sprite = self.scene.child_mut(self.piece_sprite_id).unwrap();
        Self::put_piece_sprites(self.context, piece_sprite, &piece);
    }

    fn move_piece(&mut self, piece: Piece, position: BlockIndexOffset) {
        let position = position.to_pixel_space(self.stage_size())
            - BlockIndex::new(0, 0)
                .to_pixel_space(piece.size())
                .to_vector();
        self.piece_sprite().move_to(position);
    }

    fn remove_piece(&mut self) {
        self.piece_sprite().remove_all_children();
    }

    fn update_next_pieces(&mut self, pieces: Vec<Piece>) {
        let parent = self.scene.child_mut(self.next_pieces_sprite_id).unwrap();
        parent.remove_all_children();
        let offset = 5.0 * TILE_SIZE;
        for (index, piece) in pieces.iter().enumerate() {
            let mut sprite = self
                .context
                .empty_sprite()
                .moved_to(euclid::TypedPoint2D::new(0.0, offset * index as f64));
            Self::put_piece_sprites(self.context, &mut sprite, piece);
            parent.add_child(sprite);
        }
    }

    fn put_blocks(&mut self, blocks: Vec<(Block, BlockIndex)>) {
        for (block, index) in blocks {
            if let Some(old_id) = self.block_ids[index] {
                self.stage_sprite().remove_child(old_id);
            }
            self.block_ids[index] = Some(
                sprite::Sprite::from_texture(
                    self.context.assets.block_texture(block, BlockFace::Normal),
                )
                .moved_to(index.to_pixel_space(self.stage_size()))
                .add_to(self.stage_sprite()),
            );
        }
    }

    fn add_removing_action(&mut self, block: Block, index: BlockIndex) {
        use ai_behavior::{Action, Sequence};
        use sprite::{Ease, EaseFunction, ScaleTo};
        let id = self.block_ids[index].unwrap();
        let texture = self.context.assets.block_texture(block, BlockFace::Happy);
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

    fn remove_blocks(&mut self, blocks: Vec<(Block, BlockIndex)>) {
        for &(block, index) in &blocks {
            self.add_removing_action(block, index);
        }
        self.jobs.push_front(Job::Run(Box::new(move |this| {
            for (_block, index) in blocks {
                let id = std::mem::replace(&mut this.block_ids[index], None).unwrap();
                this.stage_sprite().remove_child(id);
            }
        })));
    }

    fn move_blocks(&mut self, moves: Vec<(Block, BlockIndex, BlockIndex)>) {
        for (_block, source, destination) in moves {
            let id = std::mem::replace(&mut self.block_ids[source], None).unwrap();
            let position = destination.to_pixel_space(self.stage_size());
            self.scene.child_mut(id).unwrap().move_to(position);
            self.block_ids[destination] = Some(id);
        }
    }

    fn apply_game_event(&mut self, event: GameEvent) {
        use GameEvent::*;
        match event {
            ChangePiece(piece) => {
                self.change_piece(piece);
            }
            MovePiece(piece, position) => {
                self.move_piece(piece, position);
            }
            RemovePiece => {
                self.remove_piece();
            }
            UpdateNextPieces(pieces) => {
                self.update_next_pieces(pieces);
            }
            PutBlocks(blocks) => {
                self.put_blocks(blocks);
            }
            RemoveBlocks(blocks) => {
                self.remove_blocks(blocks);
            }
            MoveBlocks(moves) => {
                self.move_blocks(moves);
            }
        }
    }

    fn execute_jobs(&mut self) {
        loop {
            if !self.is_ready() {
                break;
            }
            if let Some(job) = self.jobs.pop_front() {
                match job {
                    Job::GameEvent(event) => {
                        self.apply_game_event(event);
                    }
                    Job::Run(f) => {
                        f(self);
                    }
                }
            } else {
                break;
            }
        }
    }

    fn apply_game_events(&mut self, events: Vec<GameEvent>) {
        self.jobs
            .extend(events.into_iter().map(|event| Job::GameEvent(event)));
        self.execute_jobs();
    }

    fn update(&mut self, delta: f64) {
        self.execute_jobs();
        if self.is_ready() {
            let events = self.game.update(delta);
            self.apply_game_events(events);
        }
    }

    fn input(&mut self, input: Input) {
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
                self.apply_game_events(events);
            }
            _ => {}
        }
    }

    pub fn apply_window_event(&mut self, event: Event, window: &mut PistonWindow) {
        self.scene.event(&event);
        match event {
            Event::Loop(Loop::Update(arg)) => {
                self.update(arg.dt);
            }
            Event::Loop(Loop::Render(_)) => {
                window.draw_2d(&event, |c, g, _| {
                    clear([0.0, 0.0, 0.0, 1.0], g);
                    self.scene.draw(c.transform, g);
                });
            }
            Event::Input(input, _) => {
                self.input(input);
            }
            _ => {}
        }
    }
}