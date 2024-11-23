use starframe as sf;

pub mod level;
pub mod physics_layers;
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
    // meshes will come from gltf eventually,
    // but it might still be nice to have them in this struct
    // so we don't have to look them up by string id
    block_mesh: sf::MeshId,
    cloud_mesh: sf::MeshId,
    player_collider: sf::Collider,
    player_mesh: sf::MeshId,
    // separate mesh with a different color for when double jump is spent
    player_mesh_doublejumped: sf::MeshId,
    bullet_collider: sf::Collider,
    bullet_mesh: sf::MeshId,
    background_mesh: sf::MeshId,
}

impl Assets {
    fn load(game: &mut sf::Game) -> Self {
        game.graphics
            .load_gltf("assets/models.glb")
            .expect("assets/models.glb not found");

        let block_collider = sf::Collider::new_square(1.);
        // sf note: would be much nicer if we had a default mesh as a fallback
        // instead of having to deal with options here
        let block_mesh = game.graphics.get_mesh_id("models.block_wood").unwrap();

        let cloud_mesh = game.graphics.create_mesh(sf::MeshParams {
            name: Some("cloud"),
            data: sf::MeshData::from(block_collider),
            ..Default::default()
        });
        let cloud_material = game.graphics.create_material(sf::MaterialParams {
            name: Some("block"),
            base_color: Some([0.722, 0.807, 0.820, 1.]),
            attenuation: Some(sf::AttenuationParams {
                color: [0.722, 0.807, 0.820],
                distance: 0.5,
            }),
            ..Default::default()
        });
        game.graphics.set_mesh_material(cloud_mesh, cloud_material);

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
                distance: 0.5,
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
                distance: 0.5,
            }),
            ..Default::default()
        });
        game.graphics
            .set_mesh_material(player_mesh_doublejumped, player_material_doublejumped);

        let bullet_collider = sf::Collider::new_circle(0.4);
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

        Self {
            block_mesh,
            cloud_mesh,
            player_collider,
            player_mesh,
            player_mesh_doublejumped,
            bullet_collider,
            bullet_mesh,
            background_mesh,
        }
    }
}

impl sf::GameState for State {
    fn init(game: &mut sf::Game) -> Self {
        physics_layers::setup(&mut game.physics);

        let assets = Assets::load(game);
        let mut level_gen = level::LevelGenerator::new(include_str!("level/patterns.txt"));
        level_gen.generate(game, &assets);
        player::spawn(game, &assets);

        let mut camera = sf::Camera::new();
        camera.pose.translation.x = level::LEVEL_WIDTH / 2.;
        // always scale the view to the same height
        // (this can lose sight of the level edges if the window is too narrow.
        // sf note: add a way to enforce 16:9 aspect ratio)
        camera.view_width = 1.;
        camera.view_height = level::VIEW_HEIGHT;

        let mut env_map = sf::EnvironmentMap::preset_night();
        env_map.lights.clear();

        Self {
            assets,
            level_gen,
            camera,
            env_map,
        }
    }

    fn tick(&mut self, game: &mut sf::Game) -> Option<()> {
        player::tick(game, &self.assets);
        // sf note: probably would be nicer to take a 32-bit vector for this forcefield
        // (in general the mixing of f64 and f32 is a bit unfortunate, also in collider parameters.
        // probably should take f32s in every user-facing API)
        game.physics_tick(&sf::forcefield::Gravity(sf::DVec2::new(0., -15.)), None);

        player::move_camera(game, &mut self.camera);

        player::handle_bullets(game, &self.camera);
        level::tile::break_tiles(game);

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
