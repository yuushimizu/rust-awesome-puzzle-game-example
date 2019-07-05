use crate::assets::Assets;
use crate::sprite_ext::Sprite;
use piston_window::*;

pub struct SceneContext {
    pub assets: Assets,
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

        }
    }

    pub fn empty_sprite(&self) -> Sprite {
        Sprite::from_texture(self.assets.empty_texture())
    }
}