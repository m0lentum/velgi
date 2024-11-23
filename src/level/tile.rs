use core::f32;

use rand::Rng;
use starframe as sf;

use crate::Assets;

#[derive(Clone, Copy, Debug)]
pub enum Tile {
    Empty,
    Cloud,
    GroundWeak,
    GroundStrong,
    // unbreakable ground only at the starting platform
    GroundUnbreakable,
}

/// State that tracks when a block needs to break
#[derive(Clone, Copy, Debug)]
pub struct BreakableTile {
    pub time_to_break: f32,
    pub is_breaking: bool,
    pub blocks_bullets: bool,
}

impl Tile {
    pub fn pick(c: char) -> Self {
        let mut rng = rand::thread_rng();
        // some tiles only spawn occasionally,
        // represented by the probability given here
        let (tile, chance) = match c {
            'X' => (Self::GroundStrong, 1.),
            'x' => (Self::GroundStrong, 0.5),
            'W' => (Self::GroundWeak, 1.),
            'w' => (Self::GroundWeak, 0.5),
            'C' => (Self::Cloud, 1.),
            'c' => (Self::Cloud, 0.5),
            _ => (Self::Empty, 1.),
        };

        if chance == 1. {
            return tile;
        }

        if rng.gen_bool(chance) {
            tile
        } else {
            Self::Empty
        }
    }

    pub fn time_to_break(&self) -> Option<f32> {
        match self {
            Self::GroundUnbreakable | Self::Empty => None,
            Self::GroundStrong => Some(2.),
            Self::GroundWeak => Some(0.75),
            Self::Cloud => Some(0.5),
        }
    }

    /// Spawn this tile at the given position in the grid.
    pub fn spawn(self, game: &mut sf::Game, assets: &Assets, pos: (i32, i32)) {
        if let Self::Empty = self {
            return;
        }

        // position the center of the tile in the middle of the grid space
        let ent_pos = sf::Vec2::new(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5);

        let pose = sf::PoseBuilder::new().with_position(ent_pos).build();
        let coll = sf::Collider::new_square(1.);
        let coll_key = game.physics.entity_set.insert_collider(coll);
        let mesh_id = match self {
            Self::GroundUnbreakable | Self::GroundStrong | Self::GroundWeak => assets.block_mesh,
            Self::Cloud => assets.cloud_mesh,
            Self::Empty => unreachable!(),
        };

        let ent = game.world.spawn((pose, coll_key, mesh_id));
        if let Some(time_to_break) = self.time_to_break() {
            let breakable = BreakableTile {
                time_to_break,
                is_breaking: false,
                blocks_bullets: !matches!(self, Self::Cloud | Self::Empty),
            };
            game.world.insert_one(ent, breakable).unwrap();
        }
    }
}

/// Check if any breakable tiles need to be removed from the game.
/// This steps each tile's timer forward by `game.dt_fixed`,
/// so call once per update.
pub fn break_tiles(game: &mut sf::Game) {
    let mut break_queue: Vec<sf::hecs::Entity> = Vec::new();
    for (ent, (tile,)) in game.world.query_mut::<(&mut BreakableTile,)>() {
        if tile.is_breaking {
            tile.time_to_break -= game.dt_fixed as f32;
            if tile.time_to_break <= 0. {
                break_queue.push(ent);
            }
        }
    }

    for ent in break_queue {
        game.world.despawn(ent).unwrap();
    }
}
