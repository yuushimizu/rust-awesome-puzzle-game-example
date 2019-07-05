use crate::game::block::Block;
use find_folder;
use piston_window;
use std::collections;
use std::path;
use std::rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BlockFace {
    Normal,
    Sleep,
    Happy,
}

impl BlockFace {
    fn name(self) -> &'static str {
        use BlockFace::*;
        match self {
            Normal => "normal",
            Sleep => "sleep",
            Happy => "happy",
        }
    }
}

type TextureContext = piston_window::G2dTextureContext;

pub type Texture = piston_window::G2dTexture;

pub struct Assets {
    path: path::PathBuf,
    texture_context: piston_window::G2dTextureContext,
    texture_settings: piston_window::TextureSettings,
    textures: collections::HashMap<String, rc::Rc<Texture>>,
    empty_texture: rc::Rc<Texture>,
}

impl Assets {
    pub fn new(
        mut texture_context: TextureContext,
        texture_settings: piston_window::TextureSettings,
    ) -> Self {
        let empty_texture = rc::Rc::new(
            Texture::empty(&mut texture_context).expect("can not create an empty texture"),
        );
        Self {
            path: find_folder::Search::ParentsThenKids(3, 3)
                .for_folder("assets")
                .expect("the assets directory was not found"),
            texture_context,
            texture_settings,
            textures: collections::HashMap::new(),
            empty_texture,
        }
    }

    fn load_texture(&mut self, name: &str) -> Texture {
        Texture::from_path(
            &mut self.texture_context,
            self.path.join(name),
            piston_window::Flip::None,
            &self.texture_settings,
        )
        .expect(&format!("can not load the texture: {}", name))
    }

    fn texture(&mut self, name: &str) -> rc::Rc<Texture> {
        if let Some(texture) = self.textures.get(name) {
            texture.clone()
        } else {
            rc::Rc::new(self.load_texture(name))
        }
    }

    fn block_texture_name(&self, block: Block, face: BlockFace) -> String {
        format!("block-{}-{}.png", block.number, face.name())
    }

    pub fn block_texture(&mut self, block: Block, face: BlockFace) -> rc::Rc<Texture> {
        self.texture(&self.block_texture_name(block, face))
    }

    pub fn background_tile_texture(&mut self) -> rc::Rc<Texture> {
        self.texture("bg-tile.png")
    }

    pub fn empty_texture(&self) -> rc::Rc<Texture> {
        self.empty_texture.clone()
    }
}