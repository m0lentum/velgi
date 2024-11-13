use starframe as sf;

pub mod level;
pub mod player;

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
            lighting_quality: sf::LightingQualityConfig::default(),
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
}

pub struct Assets {
    block_collider: sf::Collider,
    // meshes will come from gltf eventually,
    // but it might still be nice to have them in this struct
    // so we don't have to look them up by string id
    block_mesh: sf::MeshId,
    player_collider: sf::Collider,
    player_mesh: sf::MeshId,
    bullet_collider: sf::Collider,
    bullet_mesh: sf::MeshId,
    background_mesh: sf::MeshId,
}

impl Assets {
    fn load(game: &mut sf::Game) -> Self {
        let block_collider = sf::Collider::new_square(level::TILE_SIZE as f64);
        let block_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("block"),
            data: sf::MeshData::from(block_collider),
            ..Default::default()
        });
        let block_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("block"),
            base_color: Some([0.660, 0.441, 0.191, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.660, 0.441, 0.191],
                distance: 0.2,
            }),
            ..Default::default()
        });
        game.graphics.set_mesh_material(block_mesh, block_material);

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
            attenuation: Some(sf::AttenuationParams {
                color: [0.598, 0.740, 0.333],
                distance: 0.05,
            }),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(player_mesh, player_material);

        let bullet_collider = sf::Collider::new_circle(0.8);
        let bullet_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("bullet"),
            data: sf::MeshData::from(bullet_collider),
            ..Default::default()
        });
        let bullet_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("bullet"),
            base_color: Some([0.910, 0.830, 0.473, 1.]),
            emissive_color: Some([0.910, 0.830, 0.473, 1.]),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(bullet_mesh, bullet_material);

        let background_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("wall"),
            data: sf::MeshData::from(sf::Collider::new_rect(20., 20.)),
            ..Default::default()
        });
        let background_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("wall"),
            base_color: Some([0.0800, 0.0593, 0.0400, 1.]),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(background_mesh, background_material);

        Self {
            block_collider,
            block_mesh,
            player_collider,
            player_mesh,
            bullet_collider,
            bullet_mesh,
            background_mesh,
        }
    }
}

impl sf::GameState for State {
    fn init(game: &mut sf::Game) -> Self {
        let assets = Assets::load(game);
        let mut level_gen = level::LevelGenerator::new(include_str!("level/patterns.txt"));
        level_gen.generate(game, &assets);
        player::spawn(game, &assets);

        let mut camera = sf::Camera::new();
        camera.pose.translation.x = level::LEVEL_WIDTH / 2.;
        camera.view_width = level::LEVEL_WIDTH;
        camera.view_height = level::LEVEL_WIDTH;
        let env_map = sf::EnvironmentMap::preset_day();

        Self {
            assets,
            level_gen,
            camera,
            env_map,
        }
    }

    fn tick(&mut self, game: &mut sf::Game) -> Option<()> {
        player::tick(game);
        // sf note: probably would be nicer to take a 32-bit vector for this forcefield
        // (in general the mixing of f64 and f32 is a bit unfortunate)
        game.physics_tick(&sf::forcefield::Gravity(sf::DVec2::new(0., -15.)), None);

        player::move_camera(game, &mut self.camera);

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
