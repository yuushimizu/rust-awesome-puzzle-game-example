mod assets;
mod game;

use assets::{Assets, BlockFace};
use euclid;
use piston_window;
use game::Block;
use sprite;

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
    let mut sp =
        sprite::Sprite::from_texture(assets.block_texture(Block::new(0), BlockFace::Happy));
    sp.set_position(100.0, 100.0);
    sp.set_scale(4.0, 4.0);
    scene.add_child(sp);
    while let Some(event) = window.next() {
        window.draw_2d(&event, |c, g, _| {
            piston_window::clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);
        });
    }
}
