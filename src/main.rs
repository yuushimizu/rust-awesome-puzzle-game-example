mod assets;
mod game;
mod game_scene;
mod scene_context;
mod sprite_ext;

use euclid;
use game_scene::GameScene;
use piston_window::*;
use scene_context::SceneContext;

enum WindowSpace {}

type WindowSize = euclid::TypedSize2D<f64, WindowSpace>;

const PIXEL_SCALE: f64 = 2.0;

fn main() {
    let window_size = WindowSize::new(320.0, 320.0);
    let mut window: PistonWindow =
        WindowSettings::new("( o_o)", (window_size.width, window_size.height))
            .resizable(false)
            .automatic_close(true)
            .build()
            .expect("failed to start the game");
    window.set_max_fps(15);
    let mut scene_context = SceneContext::new(&mut window);
    scene_context.root().set_scale(PIXEL_SCALE, PIXEL_SCALE);
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
