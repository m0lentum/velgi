use starframe as sf;

use crate::{level::tile::BreakableTile, player::PlayerState};

pub struct SpikeRoller {
    entity: sf::hecs::Entity,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TickResult {
    pub player_hit: bool,
}

impl SpikeRoller {
    pub fn spawn(game: &mut sf::Game, assets: &crate::Assets) -> Self {
        let pose = sf::PoseBuilder::new()
            .with_position([
                crate::level::LEVEL_WIDTH / 2.,
                -crate::level::VIEW_HEIGHT / 2. + 0.5,
            ])
            .build();
        // body needed to get events for collisions with static tiles
        let body = sf::Body::new_kinematic();
        let body = game.physics.entity_set.insert_body(body);
        let coll = sf::Collider::new_rect(crate::level::LEVEL_WIDTH as f64, 1.)
            .with_layer(crate::physics_layers::SPIKE_ROLLER)
            .sensor();
        let coll = game.physics.entity_set.attach_collider(body, coll);
        let mesh = assets.spike_roller_mesh;

        let entity = game.world.spawn((pose, coll, mesh));

        Self { entity }
    }

    pub fn tick(
        &self,
        game: &mut sf::Game,
        camera: &sf::Camera,
        player: &PlayerState,
    ) -> TickResult {
        let Ok((pose, coll)) = game
            .world
            .query_one_mut::<(&mut sf::Pose, &sf::ColliderKey)>(self.entity)
        else {
            return TickResult::default();
        };

        let target_height = camera.pose.translation.y - crate::level::VIEW_HEIGHT / 2. + 0.5;
        if target_height > pose.translation.y {
            pose.translation.y = target_height;
            // TODO: animate if moved enough over the past few frames
            // (probably won't have time for this)
        }

        // check for collisions with tiles and player, destroy them

        for cont in game.physics.contacts_for_collider(*coll) {
            let Some(ent) = game.hecs_sync.get_collider_entity(cont.colliders[1]) else {
                continue;
            };

            if let Ok((tile,)) = game.world.query_one_mut::<(&mut BreakableTile,)>(ent) {
                tile.is_breaking = true;
                tile.time_to_break = tile.time_to_break.min(0.25);
            } else if ent == player.entity {
                return TickResult { player_hit: true };
            } else {
                // all entities besides the player and tiles are just destroyed
                game.world.despawn(ent).ok();
            }
        }

        TickResult::default()
    }
}
