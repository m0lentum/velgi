use itertools::Itertools;
use rand::seq::SliceRandom;
use starframe as sf;

pub mod tile;
use tile::Tile;

pub const TILEMAP_WIDTH: i32 = 20;
pub const LEVEL_WIDTH: f32 = TILEMAP_WIDTH as f32;
/// Height of a single pattern / "floor" measured in tiles
pub const CHUNK_HEIGHT: i32 = 8;
/// Height of an entire level measured in chunks of CHUNK_HEIGHT tiles
pub const LEVEL_HEIGHT: i32 = 30;
/// Height seen on camera at any given time
pub const VIEW_HEIGHT: f32 = 14.;

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
        // starting platforms
        for height in -4..0 {
            Tile::GroundUnbreakable.spawn(game, assets, (0, height));
            Tile::GroundUnbreakable.spawn(game, assets, (TILEMAP_WIDTH - 1, height));
        }

        // background and side walls
        let chunk_height = CHUNK_HEIGHT as f32;
        for chunk_idx in -1..LEVEL_HEIGHT + 1 {
            let halfway_width = LEVEL_WIDTH / 2.;
            let mid_height = (chunk_idx as f32 + 0.5) * chunk_height;
            let pose = sf::PoseBuilder::new()
                .with_position([halfway_width, mid_height])
                .with_depth(10.)
                .build();
            let mesh = assets.background_mesh;
            game.world.spawn((pose, mesh));

            let side_wall_coll = sf::Collider::new_square(CHUNK_HEIGHT as f64);
            let left_wall_x = -CHUNK_HEIGHT as f32 / 2.;
            let right_wall_x = TILEMAP_WIDTH as f32 + CHUNK_HEIGHT as f32 / 2.;
            for x in [left_wall_x, right_wall_x] {
                let pose = sf::PoseBuilder::new()
                    .with_position([x, mid_height])
                    .build();
                let coll = game.physics.entity_set.insert_collider(side_wall_coll);
                game.world.spawn((pose, coll));
            }
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
