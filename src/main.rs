use starframe as sf;

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
            lighting_quality: sf::LightingQualityConfig::MEDIUM,
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

pub struct Assets {
    // meshes will come from gltf eventually,
    // but it might still be nice to have them in this struct
    // so we don't have to look them up by string id
    block_wood_mesh: sf::MeshId,
    block_stone_mesh: sf::MeshId,
    cloud_mesh: sf::MeshId,
    player_collider: sf::Collider,
    player_mesh: sf::MeshId,
    // separate mesh with a different color for when double jump is spent
    player_mesh_doublejumped: sf::MeshId,
    bullet_mesh: sf::MeshId,
    background_mesh: sf::MeshId,
    spike_roller_mesh: sf::MeshId,
    bomb_mesh: sf::MeshId,
    lantern_mesh: sf::MeshId,
}

impl Assets {
    fn load(game: &mut sf::Game) -> Self {
        game.graphics
            .load_gltf_bytes("models", include_bytes!("../assets/models.glb"))
            .expect("failed to load assets");

        // sf note: would be much nicer if we had a default mesh as a fallback
        // instead of having to deal with options here
        let block_wood_mesh = game.graphics.get_mesh_id("models.block_wood").unwrap();
        let block_stone_mesh = game.graphics.get_mesh_id("models.block_stone").unwrap();
        let cloud_mesh = game.graphics.get_mesh_id("models.block_cloud").unwrap();

        let player_collider =
            sf::Collider::new_rounded_rect(0.8, 1., 0.1).with_material(sf::PhysicsMaterial {
                static_friction_coef: None,
                dynamic_friction_coef: None,
                restitution_coef: 0.,
            });
        let player_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("player"),
            data: sf::MeshData::from(player_collider),
            ..Default::default()
        });
        let player_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("player"),
            base_color: Some([0.598, 0.740, 0.333, 1.]),
            emissive_color: Some([0.598, 0.740, 0.333, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.598, 0.740, 0.333],
                distance: 0.25,
            }),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(player_mesh, player_material);

        let player_mesh_doublejumped = game.graphics.create_mesh(sf::MeshParams {
            name: Some("player"),
            data: sf::MeshData::from(player_collider),
            ..Default::default()
        });
        let player_material_doublejumped = game.graphics.create_material(sf::MaterialParams {
            name: Some("player doublejump spent"),
            base_color: Some([0.700, 0.368, 0.161, 1.]),
            emissive_color: Some([0.700, 0.368, 0.161, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.700, 0.368, 0.161],
                distance: 0.25,
            }),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(player_mesh_doublejumped, player_material_doublejumped);

        let bullet_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("bullet"),
            data: sf::MeshData::from(sf::Collider::new_circle(0.4)),
            ..Default::default()
        });
        let bullet_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("bullet"),
            base_color: Some([0.910, 0.830, 0.473, 1.]),
            emissive_color: Some([0.910, 0.830, 0.473, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.910, 0.830, 0.473],
                distance: 0.25,
            }),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(bullet_mesh, bullet_material);

        let bomb_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("enemy"),
            data: sf::MeshData::from(sf::Collider::new_circle(0.4)),
            ..Default::default()
        });
        let bomb_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("enemy"),
            base_color: Some([0.930, 0.298, 0.140, 1.]),
            emissive_color: Some([0.930, 0.298, 0.140, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.930, 0.298, 0.140],
                distance: 0.25,
            }),
            ..Default::default()
        });
        game.graphics.set_mesh_material(bomb_mesh, bomb_material);

        let background_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("wall"),
            data: sf::MeshData::from(sf::Collider::new_rect(
                level::TILEMAP_WIDTH as f64,
                level::CHUNK_HEIGHT as f64,
            )),
            ..Default::default()
        });
        let background_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("wall"),
            base_color: Some([0.227, 0.265, 0.420, 1.]),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(background_mesh, background_material);

        let spike_roller_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("spike_roller"),
            data: sf::MeshData::from(sf::Collider::new_rect(level::TILEMAP_WIDTH as f64, 1.)),
            ..Default::default()
        });
        let spike_roller_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("spike_roller"),
            base_color: Some([0.722, 0.807, 0.820, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.722, 0.809, 0.820],
                distance: 0.1,
            }),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(spike_roller_mesh, spike_roller_material);

        let lantern_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("lantern"),
            data: sf::MeshData::from(sf::Collider::new_capsule(0.75, 0.5)),
            ..Default::default()
        });
        // sf note: with the current volumetrics impl
        // lights have to be pretty big to be bright.
        // opaque lights should have a special treatment
        // where they emit all their light immediately instead of over distance
        let lantern_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("lantern"),
            base_color: Some([0.990, 0.973, 0.782, 0.5]),
            emissive_color: Some([0.990, 0.973, 0.782, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.5, 0.5, 0.5],
                distance: 0.1,
            }),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(lantern_mesh, lantern_material);

        Self {
            block_wood_mesh,
            block_stone_mesh,
            cloud_mesh,
            player_collider,
            player_mesh,
            player_mesh_doublejumped,
            bullet_mesh,
            background_mesh,
            spike_roller_mesh,
            bomb_mesh,
            lantern_mesh,
        }
    }
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
                // sf note: would be nicer to take a 32-bit vector for this forcefield
                // (in general the mixing of f64 and f32 is a bit unfortunate, also in collider parameters.
                // probably should take f32s in every user-facing API)
                game.physics_tick(&sf::forcefield::Gravity(sf::DVec2::new(0., -15.)), None);

                self.player.move_camera(game, &mut self.camera);
                let roller_result = self.spike_roller.tick(game, &self.camera, &self.player);

                player::handle_bullets(game, &self.camera);
                level::tile::break_tiles(game);

                if roller_result.player_hit {
                    self.state = GameplayState::GameOver;
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
