use rand::seq::SliceRandom;
use starframe as sf;

pub const TILEMAP_WIDTH: usize = 20;
pub const TILE_SIZE: f32 = 1.;
pub const LEVEL_WIDTH: f32 = TILEMAP_WIDTH as f32 * TILE_SIZE;

pub struct TileGenerator {
    patterns: Vec<String>,
    /// number of rows already generated
    curr_height: usize,
}

impl TileGenerator {
    pub fn new(pattern_data: &str) -> Self {
        let patterns = pattern_data
            .split("\n\n")
            .map(|s| String::from_iter(s.chars().filter(|c| *c != '\n')))
            .collect();

        Self {
            patterns,
            curr_height: 0,
        }
    }

    pub fn gen_until(&mut self, game: &mut sf::Game, assets: &super::Assets, row: usize) {
        let mut rng = rand::thread_rng();
        while self.curr_height <= row {
            let pat = self.patterns.choose(&mut rng).unwrap();
            let pat_height = pat.len() / TILEMAP_WIDTH;
            // add one empty row between all the patterns
            self.curr_height += pat_height + 1;

            for (i, c) in pat.chars().enumerate() {
                let tile_x = i % TILEMAP_WIDTH;
                let tile_y = self.curr_height - i / TILEMAP_WIDTH;
                // half offset to the tile center
                let tile_pos = sf::Vec2::new(tile_x as f32 + 0.5, tile_y as f32 + 0.5) * TILE_SIZE;

                let tile = Tile::pick(c);
                tile.spawn(game, assets, tile_pos);
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
}

impl Tile {
    fn pick(c: char) -> Self {
        match c {
            // TODO: random chance to spawn on lowercase letters
            'X' | 'x' => Self::GroundStrong,
            'W' | 'w' => Self::GroundWeak,
            'C' | 'c' => Self::Cloud,
            _ => Self::Empty,
        }
    }

    fn spawn(self, game: &mut sf::Game, assets: &super::Assets, pos: sf::Vec2) {
        if let Self::Empty = self {
            return;
        }
        let pose = sf::PoseBuilder::new().with_position(pos).build();
        let coll = assets.block_collider;
        let coll_key = game.physics.entity_set.insert_collider(coll);
        // TODO pick graphic based on type
        let mesh_id = assets.block_mesh;

        game.world.spawn((pose, coll_key, mesh_id));
    }
}
