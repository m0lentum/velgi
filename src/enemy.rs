use starframe as sf;

use crate::{player::PlayerState, Assets};

const BAT_CHASE_SPEED: f64 = 3.;
const BAT_SPOT_RANGE: f32 = 8.;

#[derive(Clone, Copy, Debug)]
pub enum Enemy {
    Bat { is_active: bool },
}

impl Enemy {
    pub fn bat() -> Self {
        Self::Bat { is_active: false }
    }

    pub fn spawn(&self, game: &mut sf::Game, assets: &Assets, pos: sf::Vec2) {
        let pose = sf::PoseBuilder::new().with_position(pos).build();
        let body = sf::Body::new_particle(1.).ignore_gravity();
        let body = game.physics.entity_set.insert_body(body);
        let coll = sf::Collider::new_circle(0.25)
            // sf note: this should be available as a preset
            // because it's very common in games
            .with_material(sf::PhysicsMaterial {
                static_friction_coef: None,
                dynamic_friction_coef: None,
                restitution_coef: 0.,
            })
            .with_layer(crate::physics_layers::ENEMY);
        let coll = game.physics.entity_set.attach_collider(body, coll);
        let mesh = assets.bomb_mesh;

        game.world.spawn((*self, pose, body, coll, mesh));
    }

    pub fn tick(game: &mut sf::Game, player: &PlayerState) {
        let Ok((&player_pose,)) = game.world.query_one_mut::<(&sf::Pose,)>(player.entity) else {
            return;
        };

        for (_, (enemy, bat_pose, body_key)) in game
            .world
            .query_mut::<(&mut Self, &sf::Pose, &sf::BodyKey)>()
        {
            match enemy {
                Self::Bat { is_active } => {
                    let Some(body) = game.physics.entity_set.get_body_mut(*body_key) else {
                        continue;
                    };
                    let to_player = player_pose.translation.xy() - bat_pose.translation.xy();
                    let dist_to_player = to_player.mag();
                    let dir_to_player = to_player / dist_to_player;
                    if *is_active {
                        // has already seen the player, chase them
                        body.velocity.linear.x = dir_to_player.x as f64 * BAT_CHASE_SPEED;
                        body.velocity.linear.y = dir_to_player.y as f64 * BAT_CHASE_SPEED;
                    } else if dist_to_player <= BAT_SPOT_RANGE {
                        // check for line of sight
                        // sf note: I'd like this to ignore clouds
                        // but that's currently not possible,
                        // add an API for raycasting with collision layers
                        let Some(hit) = game.physics.raycast(
                            sf::Ray {
                                start: sf::DVec2::new(
                                    bat_pose.translation.x as f64,
                                    bat_pose.translation.y as f64,
                                ),
                                dir: sf::math::UnitDVec2::new_unchecked(sf::DVec2::new(
                                    dir_to_player.x as f64,
                                    dir_to_player.y as f64,
                                )),
                            },
                            dist_to_player as f64,
                        ) else {
                            continue;
                        };

                        if game
                            .physics
                            .entity_set
                            .get_collider(hit.collider)
                            .unwrap()
                            .layer
                            == crate::physics_layers::PLAYER
                        {
                            // player was the first thing hit -> is in line of sight
                            *is_active = true;
                        }
                    }
                }
            }
        }
    }
}
