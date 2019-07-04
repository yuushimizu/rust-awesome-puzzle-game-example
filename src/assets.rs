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

type Texture = piston_window::G2dTexture;

pub struct Assets {
    path: path::PathBuf,
    texture_context: piston_window::G2dTextureContext,
    texture_settings: piston_window::TextureSettings,
    textures: collections::HashMap<path::PathBuf, rc::Rc<Texture>>,
}

impl Assets {
    pub fn new(
        texture_context: TextureContext,
        texture_settings: piston_window::TextureSettings,
    ) -> Self {
        Self {
            path: find_folder::Search::ParentsThenKids(3, 3)
                .for_folder("assets")
                .expect("the assets directory was not found"),
            texture_context,
            texture_settings,
            textures: collections::HashMap::new(),
        }
    }

    fn load_texture<P: AsRef<path::Path>>(&mut self, path: P) -> Texture {
        Texture::from_path(
            &mut self.texture_context,
            path.as_ref(),
            piston_window::Flip::None,
            &self.texture_settings,
        )
        .expect(&format!(
            "can not load the texture: {}",
            path.as_ref().display()
        ))
    }

    fn texture<P: AsRef<path::Path>>(&mut self, path: P) -> rc::Rc<Texture> {
        if let Some(texture) = self.textures.get(path.as_ref()) {
            texture.clone()
        } else {
            rc::Rc::new(self.load_texture(path))
        }
    }

    fn block_path(&self, block: &Block, face: BlockFace) -> path::PathBuf {
        self.path
            .join(format!("block-{}-{}.png", block.number, face.name()))
    }

    pub fn block_texture(&mut self, block: &Block, face: BlockFace) -> rc::Rc<Texture> {
        self.texture(self.block_path(block, face))
    }
}