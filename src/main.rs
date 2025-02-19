use starframe as sf;

pub mod assets;
pub use assets::Assets;
pub mod enemy;
use enemy::Enemy;
pub mod level;
pub mod physics_layers;
pub mod player;
use player::PlayerState;
pub mod spike_roller;
use spike_roller::SpikeRoller;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = sf::winit::window::WindowBuilder::new()
        .with_title("velgi")
        .with_inner_size(sf::winit::dpi::LogicalSize {
            width: 1280.0,
            height: 720.0,
        });

    sf::Game::run::<State>(sf::GameParams {
        window,
        graphics: sf::GraphicsConfig {
            fps: 60,
            use_vsync: false,
            lighting: sf::LightingConfig {
                quality: sf::LightingQualityConfig::HIGH,
                ..Default::default()
            },
        },
        ..Default::default()
    })?;

    Ok(())
}

pub struct State {
    assets: Assets,
    level_gen: level::LevelGenerator,
    camera: sf::Camera,
    env_map: sf::EnvironmentMap,
    state: GameplayState,
    player: PlayerState,
    spike_roller: SpikeRoller,
}

#[derive(Clone, Copy, Debug)]
pub enum GameplayState {
    Playing,
    GameOver,
}

impl State {
    fn reset(&mut self, game: &mut sf::Game) {
        // sf note: game.clear_state probably shouldn't automatically clear graphics too
        game.world.clear();
        game.physics.clear();
        game.hecs_sync.clear();
        self.level_gen.generate(game, &self.assets);
        self.camera.pose.translation.y = 3.;
        self.player = PlayerState::spawn(game, &self.assets);
        self.spike_roller = SpikeRoller::spawn(game, &self.assets);
    }
}

impl sf::GameState for State {
    fn init(game: &mut sf::Game) -> Self {
        physics_layers::setup(&mut game.physics);

        let assets = Assets::load(game);
        let mut level_gen = level::LevelGenerator::new(include_str!("level/patterns.txt"));
        level_gen.generate(game, &assets);

        let mut camera = sf::Camera::new();
        camera.pose.translation.x = level::LEVEL_WIDTH / 2.;
        camera.pose.translation.y = 3.;
        // always scale the view to the same height
        // (this can lose sight of the level edges if the window is too narrow.
        // sf note: add a way to enforce 16:9 aspect ratio)
        camera.view_width = 1.;
        camera.view_height = level::VIEW_HEIGHT;

        let mut env_map = sf::EnvironmentMap::preset_night();
        env_map.lights.clear();
        env_map.ambient.iter_mut().for_each(|c| *c *= 3.);

        let player = PlayerState::spawn(game, &assets);
        let spike_roller = SpikeRoller::spawn(game, &assets);

        Self {
            assets,
            level_gen,
            camera,
            state: GameplayState::Playing,
            env_map,
            player,
            spike_roller,
        }
    }

    fn tick(&mut self, game: &mut sf::Game) -> Option<()> {
        // keyboard controls to change lighting quality (no time to implement a settings menu)
        if game.input.button(sf::Key::Digit1.into()) {
            game.renderer
                .set_lighting_quality(sf::LightingQualityConfig::LOWEST);
        }
        if game.input.button(sf::Key::Digit2.into()) {
            game.renderer
                .set_lighting_quality(sf::LightingQualityConfig::LOW);
        }
        if game.input.button(sf::Key::Digit3.into()) {
            game.renderer
                .set_lighting_quality(sf::LightingQualityConfig::MEDIUM);
        }
        if game.input.button(sf::Key::Digit4.into()) {
            game.renderer
                .set_lighting_quality(sf::LightingQualityConfig::HIGH);
        }

        match self.state {
            GameplayState::Playing => {
                self.player.tick(game, &self.assets);
                Enemy::tick(game, &self.player);
                game.physics_tick(&sf::forcefield::Gravity(sf::DVec2::new(0., -15.)), None);

                self.player.move_camera(game, &mut self.camera);
                let roller_result = self.spike_roller.tick(game, &self.camera, &self.player);

                player::handle_bullets(game, &self.camera);
                level::tile::break_tiles(game);

                if roller_result.player_hit {
                    self.state = GameplayState::GameOver;
                    // spawn a "game over" message in the world
                    // (we don't have text/menu type stuff in starframe yet)
                    let pose = sf::PoseBuilder::new()
                        .with_position(self.camera.pose.translation.xy())
                        .with_depth(-10.)
                        .build();
                    game.world.spawn((pose, self.assets.game_over_mesh));
                }
            }
            GameplayState::GameOver => {
                if game.input.button(sf::ButtonQuery::kb(sf::Key::ShiftLeft)) {
                    self.reset(game);
                    self.state = GameplayState::Playing;
                }
            }
        }

        Some(())
    }

    fn draw(&mut self, game: &mut sf::Game, dt: f32) {
        self.camera.upload();
        game.graphics.update_animations(dt);
        game.renderer.set_environment_map(&self.env_map);

        let mut frame = game.renderer.begin_frame();
        frame.set_clear_color([0.00802, 0.0137, 0.02732, 1.]);
        frame.draw_meshes(&mut game.graphics, &mut game.world, &self.camera);
    }
}
