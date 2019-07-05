mod assets;
mod game;

use assets::{Assets, BlockFace};
use euclid;
use euclid_ext::{Map2D, Points};
use game::{BlockPosition, Game};
use piston_window::*;
use uuid;

type Sprite = sprite::Sprite<assets::Texture>;

fn empty_sprite(assets: &Assets) -> Sprite {
    Sprite::from_texture(assets.empty_texture())
}

enum WindowSpace {}

type WindowSize = euclid::TypedSize2D<f64, WindowSpace>;

const PIXEL_SCALE: f64 = 2.0;

const TILE_SIZE: f64 = 8.0;

enum PixelSpace {}

fn tile_position(index: &BlockPosition) -> euclid::TypedPoint2D<f64, PixelSpace> {
    index.map(|n| euclid::Length::new(n.get() as f64 * TILE_SIZE))
}

struct Scene {
    scene: sprite::Scene<assets::Texture>,
    stage_id: uuid::Uuid,
    piece_id: uuid::Uuid,
}

impl Scene {
    fn new(assets: &mut Assets, game: &Game) -> Self {
        let mut scene = sprite::Scene::new();
        let mut root = empty_sprite(assets);
        root.set_scale(PIXEL_SCALE, PIXEL_SCALE);
        let mut stage = empty_sprite(assets);
        stage.set_position(100.0, 50.0);
        for index in euclid::TypedRect::new(BlockPosition::zero(), game.stage_size()).points() {
            let texture = assets.background_tile_texture();
            let mut tile = Sprite::from_texture(texture.clone());
            let position = tile_position(&index);
            tile.set_position(position.x, position.y);
            stage.add_child(tile);
        }
        let stage_id = stage.id();
        let piece = empty_sprite(assets);
        let piece_id = piece.id();
        stage.add_child(piece);
        root.add_child(stage);
        scene.add_child(root);
        Self {
            scene,
            stage_id,
            piece_id,
        }
    }

    fn stage_mut(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.stage_id).unwrap()
    }

    fn piece_mut(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.piece_id).unwrap()
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
    window.set_max_fps(30);
    let mut texture_settings = TextureSettings::new();
    texture_settings.set_filter(Filter::Nearest);
    let mut assets = Assets::new(
        TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into(),
        },
        texture_settings,
    );
    let mut game = game::Game::new();
    let mut scene = Scene::new(&mut assets, &game);
    let piece = scene.piece_mut();
    for index in euclid::TypedRect::new(BlockPosition::zero(), game.piece().size()).points() {
        if let Some(block) = game.piece().blocks()[index] {
            let texture = assets.block_texture(block, BlockFace::Normal);
            let mut sprite = sprite::Sprite::from_texture(texture.clone());
            let position = tile_position(&index);
            sprite.set_position(position.x, position.y);
            piece.add_child(sprite);
        }
    }
    while let Some(event) = window.next() {
        match event {
            Event::Loop(Loop::Render(_)) => {
                window.draw_2d(&event, |c, g, _| {
                    clear([0.0, 0.0, 0.0, 1.0], g);
                    scene.scene.draw(c.transform, g);
                });
            }
            Event::Loop(Loop::Update(arg)) => {
                game.update(arg.dt);
            }
            _ => {}
        }
    }
}
