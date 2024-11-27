use starframe as sf;

pub struct Assets {
    // meshes will come from gltf eventually,
    // but it might still be nice to have them in this struct
    // so we don't have to look them up by string id
    pub block_wood_mesh: sf::MeshId,
    pub block_stone_mesh: sf::MeshId,
    pub cloud_mesh: sf::MeshId,
    pub player_collider: sf::Collider,
    pub player_mesh: sf::MeshId,
    // separate mesh with a different color for when double jump is spent
    pub player_mesh_doublejumped: sf::MeshId,
    pub bullet_mesh: sf::MeshId,
    pub background_mesh: sf::MeshId,
    pub spike_roller_mesh: sf::MeshId,
    pub bomb_mesh: sf::MeshId,
    pub lantern_mesh: sf::MeshId,
    pub you_win_mesh: sf::MeshId,
    pub game_over_mesh: sf::MeshId,
    pub barbut_mesh: sf::MeshId,
}

impl Assets {
    pub fn load(game: &mut sf::Game) -> Self {
        game.graphics
            .load_gltf_bytes("models", include_bytes!("../assets/models.glb"))
            .expect("failed to load assets");

        // sf note: would be much nicer if we had a default mesh as a fallback
        // instead of having to deal with options here
        let block_wood_mesh = game.graphics.get_mesh_id("models.block_wood").unwrap();
        let block_stone_mesh = game.graphics.get_mesh_id("models.block_stone").unwrap();
        let cloud_mesh = game.graphics.get_mesh_id("models.block_cloud").unwrap();
        let you_win_mesh = game.graphics.get_mesh_id("models.you_win").unwrap();
        let game_over_mesh = game.graphics.get_mesh_id("models.game_over").unwrap();
        let barbut_mesh = game.graphics.get_mesh_id("models.barbut").unwrap();

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
                crate::level::TILEMAP_WIDTH as f64,
                crate::level::CHUNK_HEIGHT as f64,
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
            data: sf::MeshData::from(sf::Collider::new_rect(
                crate::level::TILEMAP_WIDTH as f64,
                1.,
            )),
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
            you_win_mesh,
            game_over_mesh,
            barbut_mesh,
        }
    }
}
