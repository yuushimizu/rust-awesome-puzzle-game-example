use crate::assets;
use euclid;
use sprite;
use uuid;

pub enum PixelSpace {}

pub type PixelPosition = euclid::TypedPoint2D<f64, PixelSpace>;

pub type Sprite = sprite::Sprite<assets::Texture>;

pub trait MoveTo {
    fn move_to(&mut self, position: PixelPosition);
}

impl MoveTo for Sprite {
    fn move_to(&mut self, position: PixelPosition) {
        self.set_position(position.x, position.y);
    }
}

pub trait MovedTo {
    fn moved_to(self, position: PixelPosition) -> Self;
}

impl MovedTo for Sprite {
    fn moved_to(mut self, position: PixelPosition) -> Self {
        self.move_to(position);
        self
    }
}

pub trait AddTo {
    fn add_to(self, parent: &mut Sprite) -> uuid::Uuid;
}

impl AddTo for Sprite {
    fn add_to(self, parent: &mut Sprite) -> uuid::Uuid {
        parent.add_child(self)
    }
}

pub trait RemoveAllChildren {
    fn remove_all_children(&mut self);
}

impl RemoveAllChildren for Sprite {
    fn remove_all_children(&mut self) {
        for id in self.children().iter().map(|c| c.id()).collect::<Vec<_>>() {
            self.remove_child(id);
        }
    }
}