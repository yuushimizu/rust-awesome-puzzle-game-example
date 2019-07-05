use crate::assets::{Assets, Texture};
use crate::sprite_ext::Sprite;
use piston_window::*;
use uuid;

pub struct SceneContext {
    pub assets: Assets,
    pub scene: sprite::Scene<Texture>,
}

impl SceneContext {
    pub fn new(window: &mut PistonWindow) -> Self {
        let mut texture_settings = TextureSettings::new();
        texture_settings.set_filter(Filter::Nearest);
        Self {
            assets: Assets::new(
                TextureContext {
                    factory: window.factory.clone(),
                    encoder: window.factory.create_command_buffer().into(),
                },
                texture_settings,
            ),
            scene: sprite::Scene::new(),
        }
    }

    pub fn child_mut(&mut self, id: uuid::Uuid) -> Option<&mut Sprite> {
        self.scene.child_mut(id)
    }

    pub fn empty_sprite(&self) -> Sprite {
        Sprite::from_texture(self.assets.empty_texture())
    }
}