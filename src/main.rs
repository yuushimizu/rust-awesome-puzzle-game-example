mod assets;
mod game;
mod game_scene;
mod scene_context;
mod sprite_ext;

use game_scene::GameScene;
use piston_window::*;
use scene_context::SceneContext;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("( o_o)", (320, 320))
        .resizable(false)
        .automatic_close(true)
        .build()
        .expect("failed to start the game");
    window.set_max_fps(15);
    let mut scene_context = SceneContext::new(&mut window);
    let mut game_scene = GameScene::new(&mut scene_context);
    while let Some(event) = window.next() {
        game_scene.apply_window_event(event, &mut window);
    }
}
