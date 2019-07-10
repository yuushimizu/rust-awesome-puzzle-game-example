use crate::assets::{BlockFace, Texture};
use crate::game::{
    BlockGridSize, BlockIndex, BlockIndexOffset, BlockSpace, Game, GameEvent, MoveResult,
    Piece, PutResult, RemoveResult,
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

struct Sprites {
    pub scene: sprite::Scene<Texture>,
    stage_sprite_id: uuid::Uuid,
    piece_sprite_id: uuid::Uuid,
    piece_guide_sprite_id: uuid::Uuid,
    next_pieces_sprite_id: uuid::Uuid,
    block_sprite_ids: array2d::Array2D<Option<uuid::Uuid>, BlockSpace>,
}

impl Sprites {
    pub fn new(stage_size: BlockGridSize, context: &mut SceneContext) -> Self {
        let mut scene = sprite::Scene::new();
        let mut root_sprite = context.empty_sprite();
        root_sprite.set_scale(SCALE, SCALE);
        let mut stage_sprite = context
            .empty_sprite()
            .moved_to(PixelPosition::new(TILE_SIZE * 4.0, 0.0));
        put_background_tile_sprites(&mut stage_sprite, stage_size, context);
        let piece_guide_sprite_id = stage_sprite.add_child(context.empty_sprite());
        let piece_sprite_id = stage_sprite.add_child(context.empty_sprite());
        let stage_sprite_id = root_sprite.add_child(stage_sprite);
        let mut next_pieces_sprite = context.empty_sprite().moved_to(PixelPosition::new(
            TILE_SIZE * (stage_size.width as f64 + 5.5),
            TILE_SIZE * 1.0,
        ));
        next_pieces_sprite.set_scale(0.75, 0.75);
        put_background_tile_sprites(
            &mut next_pieces_sprite,
            BlockGridSize::new(6, 6 * 3),
            context,
        );
        let next_pieces_sprite_id = root_sprite.add_child(next_pieces_sprite);
        scene.add_child(root_sprite);
        Self {
            scene,
            piece_sprite_id,
            piece_guide_sprite_id,
            stage_sprite_id,
            next_pieces_sprite_id,
            block_sprite_ids: array2d::Array2D::new(stage_size, None),
        }
    }

    pub fn sprite(&mut self, id: uuid::Uuid) -> Option<&mut Sprite> {
        self.scene.child_mut(id)
    }

    pub fn stage_sprite(&mut self) -> &mut Sprite {
        self.sprite(self.stage_sprite_id).unwrap()
    }

    pub fn piece_sprite(&mut self) -> &mut Sprite {
        self.sprite(self.piece_sprite_id).unwrap()
    }

    pub fn piece_guide_sprite(&mut self) -> &mut Sprite {
        self.sprite(self.piece_guide_sprite_id).unwrap()
    }

    pub fn next_pieces_sprite(&mut self) -> &mut Sprite {
        self.sprite(self.next_pieces_sprite_id).unwrap()
    }

    pub fn block_sprite_id(&self, index: BlockIndex) -> Option<uuid::Uuid> {
        self.block_sprite_ids[index]
    }

    pub fn block_sprite(&mut self, index: BlockIndex) -> Option<&mut Sprite> {
        self.block_sprite_id(index)
            .and_then(move |id| self.sprite(id))
    }

    pub fn remove_block_sprite(&mut self, index: BlockIndex) {
        if let Some(id) = self.block_sprite_ids[index] {
            self.scene.remove_child(id);
            self.block_sprite_ids[index] = None;
        }
    }

    pub fn set_block_sprite_id(&mut self, index: BlockIndex, id: uuid::Uuid) {
        self.remove_block_sprite(index);
        self.block_sprite_ids[index] = Some(id);
    }

    pub fn move_block_sprite_id(&mut self, source: BlockIndex, destination: BlockIndex) {
        if let Some(id) = std::mem::replace(&mut self.block_sprite_ids[source], None) {
            self.set_block_sprite_id(destination, id);
        }
    }

    pub fn is_running(&self) -> bool {
        use euclid_ext::Points;
        for index in euclid::TypedRect::from_size(self.block_sprite_ids.size()).points() {
            if let Some(id) = self.block_sprite_ids[index] {
                if self.scene.running_for_child(id).unwrap_or(0) > 0 {
                    return true;
                }
            }
        }
        false
    }
}

pub struct GameScene<'a> {
    game: Game,
    sprites: Sprites,
    context: &'a mut SceneContext,
    jobs: collections::VecDeque<Job>,
}

impl<'a> GameScene<'a> {
    pub fn new(context: &'a mut SceneContext) -> Self {
        let mut game = Game::new();
        let sprites = Sprites::new(game.stage_size(), context);
        let initial_events = game.initial_events();
        let mut result = Self {
            game,
            sprites,
            context,
            jobs: Default::default(),
        };
        result.apply_game_events(initial_events);
        result
    }

    fn stage_size(&self) -> BlockGridSize {
        self.game.stage_size()
    }

    fn is_ready(&self) -> bool {
        !self.sprites.is_running()
    }

    fn put_piece_sprites(
        context: &mut SceneContext,
        sprite: &mut Sprite,
        piece: &Piece,
        is_ghost: bool,
    ) {
        for (index, block) in piece.blocks() {
            sprite::Sprite::from_texture(if is_ghost {
                context.assets.ghost_block_texture()
            } else {
                context.assets.block_texture(block, BlockFace::Sleep)
            })
            .moved_to(index.to_pixel_space(piece.size()))
            .add_to(sprite);
        }
    }

    fn piece_pixel_position(&self, piece: &Piece, position: BlockIndexOffset) -> PixelPosition {
        let offset = -BlockIndex::new(0, 0)
            .to_pixel_space(piece.size())
            .to_vector();
        position.to_pixel_space(self.stage_size()) + offset
    }

    fn change_piece(&mut self, piece: Piece, guide_position: BlockIndexOffset) {
        let piece_sprite = self.sprites.piece_sprite();
        piece_sprite.remove_all_children();
        Self::put_piece_sprites(self.context, piece_sprite, &piece, false);
        let guide_position = self.piece_pixel_position(&piece, guide_position);
        let piece_guide_sprite = self.sprites.piece_guide_sprite();
        piece_guide_sprite.remove_all_children();
        Self::put_piece_sprites(self.context, piece_guide_sprite, &piece, true);
        piece_guide_sprite.move_to(guide_position);
    }

    fn move_piece(
        &mut self,
        piece: Piece,
        position: BlockIndexOffset,
        guide_position: BlockIndexOffset,
    ) {
        let position = self.piece_pixel_position(&piece, position);
        self.sprites.piece_sprite().move_to(position);
        let guide_position = self.piece_pixel_position(&piece, guide_position);
        self.sprites.piece_guide_sprite().move_to(guide_position);
    }

    fn remove_piece(&mut self) {
        self.sprites.piece_sprite().remove_all_children();
        self.sprites.piece_guide_sprite().remove_all_children();
    }

    fn update_next_pieces(&mut self, pieces: Vec<Piece>) {
        let parent = self.sprites.next_pieces_sprite();
        parent.remove_all_children();
        let offset = 5.0 * TILE_SIZE;
        for (index, piece) in pieces.iter().enumerate() {
            let mut sprite = self
                .context
                .empty_sprite()
                .moved_to(euclid::TypedPoint2D::new(0.0, offset * index as f64));
            Self::put_piece_sprites(self.context, &mut sprite, piece, false);
            parent.add_child(sprite);
        }
    }

    fn put_blocks(&mut self, results: Vec<PutResult>) {
        for result in &results {
            let id = sprite::Sprite::from_texture(
                self.context
                    .assets
                    .block_texture(result.block, BlockFace::Normal),
            )
            .moved_to(result.index.to_pixel_space(self.stage_size()))
            .add_to(self.sprites.stage_sprite());
            self.sprites.set_block_sprite_id(result.index, id);
        }
    }

    fn add_removing_action(&mut self, remove_result: &RemoveResult) {
        use ai_behavior::{Action, Sequence};
        use sprite::{Ease, EaseFunction, ScaleTo};
        let id = self.sprites.block_sprite_id(remove_result.index).unwrap();
        let texture = self
            .context
            .assets
            .block_texture(remove_result.block, BlockFace::Happy);
        self.sprites.sprite(id).unwrap().set_texture(texture);
        self.sprites.scene.run(
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

    fn remove_blocks(&mut self, results: Vec<RemoveResult>) {
        for result in &results {
            self.add_removing_action(&result);
        }
        self.jobs.push_front(Job::Run(Box::new(move |this| {
            for result in results {
                this.sprites.remove_block_sprite(result.index);
            }
        })));
    }

    fn move_blocks(&mut self, results: Vec<MoveResult>) {
        for result in &results {
            let position = result.destination.to_pixel_space(self.stage_size());
            self.sprites
                .block_sprite(result.source)
                .unwrap()
                .move_to(position);
            self.sprites
                .move_block_sprite_id(result.source, result.destination);
        }
    }

    fn apply_game_event(&mut self, event: GameEvent) {
        use GameEvent::*;
        match event {
            ChangePiece {
                piece,
                guide_position,
            } => {
                self.change_piece(piece, guide_position);
            }
            MovePiece {
                piece,
                position,
                guide_position,
            } => {
                self.move_piece(piece, position, guide_position);
            }
            RemovePiece => {
                self.remove_piece();
            }
            UpdateNextPieces(pieces) => {
                self.update_next_pieces(pieces);
            }
            PutBlocks(results) => {
                self.put_blocks(results);
            }
            RemoveBlocks(results) => {
                self.remove_blocks(results);
            }
            MoveBlocks(results) => {
                self.move_blocks(results);
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
        self.sprites.scene.event(&event);
        match event {
            Event::Loop(Loop::Update(arg)) => {
                self.update(arg.dt);
            }
            Event::Loop(Loop::Render(_)) => {
                window.draw_2d(&event, |c, g, _| {
                    clear([0.0, 0.0, 0.0, 1.0], g);
                    self.sprites.scene.draw(c.transform, g);
                });
            }
            Event::Input(input, _) => {
                self.input(input);
            }
            _ => {}
        }
    }
}