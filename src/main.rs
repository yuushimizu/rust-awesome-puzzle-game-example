mod assets;
mod game;
mod sprite_ext;

use assets::{Assets, BlockFace};
use euclid;
use euclid_ext::{Map2D, Points};
use game::{BlockIndex, BlockSpace, Game, Piece, PiecePosition};
use piston_window::*;
use sprite_ext::{AddTo, MoveTo, MovedTo, PixelPosition, RemoveAllChildren, Sprite};
use uuid;

enum WindowSpace {}

type WindowSize = euclid::TypedSize2D<f64, WindowSpace>;

struct SceneContext {
    pub assets: Assets,
    pub scene: sprite::Scene<assets::Texture>,
    root_sprite_id: uuid::Uuid,
}

impl SceneContext {
    fn new(window: &mut PistonWindow) -> Self {
        let mut texture_settings = TextureSettings::new();
        texture_settings.set_filter(Filter::Nearest);
        let assets = Assets::new(
            TextureContext {
                factory: window.factory.clone(),
                encoder: window.factory.create_command_buffer().into(),
            },
            texture_settings,
        );
        let mut scene = sprite::Scene::new();
        let mut root = Sprite::from_texture(assets.empty_texture());
        root.set_scale(PIXEL_SCALE, PIXEL_SCALE);
        let root_sprite_id = root.id();
        scene.add_child(root);
        Self {
            assets,
            scene,
            root_sprite_id,
        }
    }

    fn root(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.root_sprite_id).unwrap()
    }

    fn child_mut(&mut self, id: uuid::Uuid) -> Option<&mut Sprite> {
        self.scene.child_mut(id)
    }

    fn empty_sprite(&self) -> Sprite {
        Sprite::from_texture(self.assets.empty_texture())
    }
}

const PIXEL_SCALE: f64 = 2.0;

const TILE_SIZE: f64 = 8.0;

fn tile_position(index: euclid::TypedPoint2D<isize, BlockSpace>) -> PixelPosition {
    index.map(|n| euclid::Length::new(n.get() as f64 * TILE_SIZE))
}

fn block_position(index: BlockIndex) -> PixelPosition {
    index.map(|n| euclid::Length::new(n.get() as f64 * TILE_SIZE))
}

struct GameSceneSprite {
    stage_id: uuid::Uuid,
    piece_id: uuid::Uuid,
}

impl GameSceneSprite {
    fn new(game: &Game, context: &mut SceneContext) -> Self {
        let mut stage = context
            .empty_sprite()
            .moved_to(PixelPosition::new(100.0, 50.0));
        for index in euclid::TypedRect::new(BlockIndex::zero(), game.stage_size()).points() {
            Sprite::from_texture(context.assets.background_tile_texture())
                .moved_to(block_position(index))
                .add_to(&mut stage);
        }
        Self {
            piece_id: context.empty_sprite().add_to(&mut stage),
            stage_id: stage.add_to(context.root()),
        }
    }

    fn piece_sprite<'a>(&self, context: &'a mut SceneContext) -> &'a mut Sprite {
        context.child_mut(self.piece_id).unwrap()
    }

    fn change_piece(&mut self, piece: &Piece, context: &mut SceneContext) {
        self.piece_sprite(context).remove_all_children();
        for index in euclid::TypedRect::new(BlockIndex::zero(), piece.size()).points() {
            if let Some(block) = piece.blocks()[index] {
                sprite::Sprite::from_texture(
                    context
                        .assets
                        .block_texture(block, BlockFace::Normal)
                        .clone(),
                )
                .moved_to(block_position(index))
                .add_to(self.piece_sprite(context));
            }
        }
    }

    fn move_piece(&mut self, position: PiecePosition, context: &mut SceneContext) {
        self.piece_sprite(context).move_to(tile_position(position));
    }

    fn apply_events(&mut self, events: Vec<game::Event>, context: &mut SceneContext) {
        for event in events {
            use game::Event::*;
            match event {
                ChangePiece(piece) => {
                    self.change_piece(piece, context);
                }
                MovePiece(position) => {
                    self.move_piece(position, context);
                }
            }
        }
    }
}

struct GameScene {
    game: Game,
    sprite: GameSceneSprite,
}

impl GameScene {
    fn new(context: &mut SceneContext) -> Self {
        let game = Game::new();
        let mut sprite = GameSceneSprite::new(&game, context);
        sprite.apply_events(game.initial_events(), context);
        Self { game, sprite }
    }

    fn update(&mut self, delta: f64, context: &mut SceneContext) {
        self.sprite.apply_events(self.game.update(delta), context);
    }
}

fn main() {
    let window_size = WindowSize::new(480.0, 480.0);
    let mut window: PistonWindow =
        WindowSettings::new("( o_o)", (window_size.width, window_size.height))
            .resizable(false)
            .automatic_close(true)
            .build()
            .expect("failed to start the game");
    window.set_max_fps(15);
    let mut scene_context = SceneContext::new(&mut window);
    let mut game_scene = GameScene::new(&mut scene_context);
    while let Some(event) = window.next() {
        match event {
            Event::Loop(Loop::Render(_)) => {
                window.draw_2d(&event, |c, g, _| {
                    clear([0.0, 0.0, 0.0, 1.0], g);
                    scene_context.scene.draw(c.transform, g);
                });
            }
            Event::Loop(Loop::Update(arg)) => {
                game_scene.update(arg.dt, &mut scene_context);
            }
            _ => {}
        }
    }
}
