use crate::assets::BlockFace;
use crate::game::{BlockIndex, BlockIndexOffset, Event, Game, Piece};
use crate::scene_context::SceneContext;
use crate::sprite_ext::{AddTo, MoveTo, MovedTo, PixelPosition, RemoveAllChildren, Sprite};
use euclid_ext::{Map2D, Points};
use uuid;

const TILE_SIZE: f64 = 8.0;

trait ToPixelSpace {
    type Output;

    fn to_pixel_space(self) -> Self::Output;
}

impl ToPixelSpace for BlockIndexOffset {
    type Output = PixelPosition;

    fn to_pixel_space(self) -> PixelPosition {
        self.map(|n| euclid::Length::new(n.get() as f64) * TILE_SIZE)
    }
}

impl ToPixelSpace for BlockIndex {
    type Output = PixelPosition;

    fn to_pixel_space(self) -> PixelPosition {
        self.cast::<isize>().to_pixel_space()
    }
}

struct GameSceneSprite {
    stage_id: uuid::Uuid,
    piece_id: uuid::Uuid,
}

impl GameSceneSprite {
    fn new(game: &Game, context: &mut SceneContext) -> Self {
        let mut stage = context
            .empty_sprite()
            .moved_to(PixelPosition::new(100.0, 50.0));
        for index in euclid::TypedRect::from_size(game.stage_size()).points() {
            Sprite::from_texture(context.assets.background_tile_texture())
                .moved_to(index.to_pixel_space())
                .add_to(&mut stage);
        }
        Self {
            piece_id: context.empty_sprite().add_to(&mut stage),
            stage_id: stage.add_to(context.root()),
        }
    }

    fn piece_sprite<'a>(&self, context: &'a mut SceneContext) -> &'a mut Sprite {
        context.child_mut(self.piece_id).unwrap()
    }

    fn change_piece(&mut self, piece: &Piece, context: &mut SceneContext) {
        self.piece_sprite(context).remove_all_children();
        for index in euclid::TypedRect::from_size(piece.size()).points() {
            if let Some(block) = piece.blocks()[index] {
                sprite::Sprite::from_texture(
                    context.assets.block_texture(block, BlockFace::Normal),
                )
                .moved_to(index.to_pixel_space())
                .add_to(self.piece_sprite(context));
            }
        }
    }

    fn move_piece(&mut self, position: BlockIndexOffset, context: &mut SceneContext) {
        self.piece_sprite(context)
            .move_to(position.to_pixel_space());
    }

    fn apply_events(&mut self, events: Vec<Event>, context: &mut SceneContext) {
        for event in events {
            use Event::*;
            match event {
                ChangePiece(piece) => {
                    self.change_piece(piece, context);
                }
                MovePiece(position) => {
                    self.move_piece(position, context);
                }
            }
        }
    }
}

pub struct GameScene {
    game: Game,
    sprite: GameSceneSprite,
}

impl GameScene {
    pub fn new(context: &mut SceneContext) -> Self {
        let game = Game::new();
        let mut sprite = GameSceneSprite::new(&game, context);
        sprite.apply_events(game.initial_events(), context);
        Self { game, sprite }
    }

    pub fn update(&mut self, delta: f64, context: &mut SceneContext) {
        self.sprite.apply_events(self.game.update(delta), context);
    }
}