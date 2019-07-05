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
        scene_context.scene.event(&event);
        match event {
            Event::Loop(Loop::Update(arg)) => {
                game_scene.update(arg.dt, &mut scene_context);
            }
            Event::Loop(Loop::Render(_)) => {
                window.draw_2d(&event, |c, g, _| {
                    clear([0.0, 0.0, 0.0, 1.0], g);
                    scene_context.scene.draw(c.transform, g);
                });
            }
            Event::Input(input, _) => {
                game_scene.input(input, &mut scene_context);
            }
            _ => {}
        }
    }
}
