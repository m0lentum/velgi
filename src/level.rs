use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};
use starframe as sf;

pub const TILEMAP_WIDTH: i32 = 20;
pub const TILE_SIZE: f32 = 1.;
pub const LEVEL_WIDTH: f32 = TILEMAP_WIDTH as f32 * TILE_SIZE;
/// Height of a single pattern / "floor" measured in tiles
pub const CHUNK_HEIGHT: i32 = 8;
/// Height of an entire level measured in chunks of CHUNK_HEIGHT tiles
pub const LEVEL_HEIGHT: i32 = 30;

pub struct LevelGenerator {
    patterns: Vec<String>,
}

impl LevelGenerator {
    pub fn new(pattern_data: &str) -> Self {
        let patterns = pattern_data
            .split("\n\n")
            .map(|pat| {
                #[allow(unstable_name_collisions)]
                String::from_iter(
                    pat.lines()
                        .filter(|l| !l.starts_with('#'))
                        .intersperse("\n"),
                )
            })
            .collect();

        Self { patterns }
    }

    pub fn generate(&mut self, game: &mut sf::Game, assets: &super::Assets) {
        game.world.clear();
        self.spawn_fixtures(game, assets);
        self.gen_tiles(game, assets);
    }

    /// Spawn entities that are part of the level but not given by random tile gen
    /// (player, starting platform, spike roll, etc.)
    fn spawn_fixtures(&self, game: &mut sf::Game, assets: &super::Assets) {
        for height in -4..0 {
            Tile::GroundUnbreakable.spawn(game, assets, (0, height));
            Tile::GroundUnbreakable.spawn(game, assets, (TILEMAP_WIDTH - 1, height));
        }
    }

    fn gen_tiles(&mut self, game: &mut sf::Game, assets: &super::Assets) {
        let mut rng = rand::thread_rng();
        for chunk_idx in 0..LEVEL_HEIGHT {
            // pick a pattern for the left and right sides
            // and spawn all the blocks related to each
            for (side, start_x) in [(1, 0), (-1, TILEMAP_WIDTH - 1)] {
                let pat = self.patterns.choose(&mut rng).unwrap();

                let mut tile_x = start_x;
                // start at the top of the chunk, fill downwards in order
                let mut tile_y = (chunk_idx + 1) * CHUNK_HEIGHT - 1;
                for c in pat.chars() {
                    if c == '\n' {
                        tile_y -= 1;
                        tile_x = start_x;
                        continue;
                    }

                    let tile = Tile::pick(c);
                    tile.spawn(game, assets, (tile_x, tile_y));

                    tile_x += side;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Tile {
    Empty,
    Cloud,
    GroundWeak,
    GroundStrong,
    // unbreakable ground only at the starting platform
    GroundUnbreakable,
}

impl Tile {
    fn pick(c: char) -> Self {
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

    /// Spawn this tile at the given position in the grid.
    fn spawn(self, game: &mut sf::Game, assets: &super::Assets, pos: (i32, i32)) {
        if let Self::Empty = self {
            return;
        }

        // position the center of the tile in the middle of the grid space
        let ent_pos = sf::Vec2::new(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5) * TILE_SIZE;

        let pose = sf::PoseBuilder::new().with_position(ent_pos).build();
        let coll = assets.block_collider;
        let coll_key = game.physics.entity_set.insert_collider(coll);
        // TODO pick graphic based on type
        let mesh_id = assets.block_mesh;

        game.world.spawn((pose, coll_key, mesh_id));
    }
}
