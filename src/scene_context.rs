use crate::assets::{Assets, Texture};
use crate::sprite_ext::Sprite;
use piston_window::*;
use uuid;

pub struct SceneContext {
    pub assets: Assets,
    pub scene: sprite::Scene<Texture>,
    root_sprite_id: uuid::Uuid,
}

impl SceneContext {
    pub fn new(window: &mut PistonWindow) -> Self {
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
        Self {
            root_sprite_id: scene.add_child(Sprite::from_texture(assets.empty_texture())),
            assets,
            scene,
        }
    }

    pub fn root(&mut self) -> &mut Sprite {
        self.scene.child_mut(self.root_sprite_id).unwrap()
    }

    pub fn child_mut(&mut self, id: uuid::Uuid) -> Option<&mut Sprite> {
        self.scene.child_mut(id)
    }

    pub fn empty_sprite(&self) -> Sprite {
        Sprite::from_texture(self.assets.empty_texture())
    }
}