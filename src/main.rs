mod assets;
mod game;

use assets::{Assets, BlockFace};
use euclid;
use euclid_ext::{Map2D, Points};
use game::BlockPosition;
use piston_window;
use piston_window::ImageSize;

enum WindowSpace {}

type WindowSize = euclid::TypedSize2D<f64, WindowSpace>;

fn main() {
    let window_size = WindowSize::new(480.0, 480.0);
    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("( o_o)", (window_size.width, window_size.height))
            .build()
            .expect("failed to start the game");
    let mut texture_settings = piston_window::TextureSettings::new();
    texture_settings.set_filter(piston_window::Filter::Nearest);
    let mut assets = Assets::new(
        piston_window::TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into(),
        },
        texture_settings,
    );
    let mut scene = sprite::Scene::new();
    let piece = game::Piece::standards()[2].clone();
    for position in euclid::TypedRect::new(BlockPosition::zero(), piece.size()).points() {
        if let Some(block) = &piece.blocks()[position] {
            let scale = 2.0;
            let texture = assets.block_texture(block, BlockFace::Normal);
            let mut sp = sprite::Sprite::from_texture(texture.clone());
            let texture_size = euclid::TypedSize2D::<u32, WindowSpace>::new(
                texture.get_width(),
                texture.get_height(),
            );
            let sprite_position: euclid::TypedPoint2D<f64, WindowSpace> = (position, texture_size)
                .map(|(position, size)| {
                    euclid::Length::new(100.0 + position.get() as f64 * size.get() as f64 * scale)
                });
            sp.set_position(sprite_position.x, sprite_position.y);
            sp.set_scale(scale, scale);
            scene.add_child(sp);
        }
    }
    while let Some(event) = window.next() {
        window.draw_2d(&event, |c, g, _| {
            piston_window::clear([0.0, 0.0, 0.0, 1.0], g);
            scene.draw(c.transform, g);
        });
    }
}
