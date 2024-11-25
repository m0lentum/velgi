use starframe as sf;

use crate::{enemy::Enemy, level::tile::BreakableTile};

const COLLIDER_WIDTH: f64 = 0.8;

const PLAYER_MASS: f64 = 1.;
const MAX_XSPEED: f64 = 7.;
const JUMP_YSPEED: f64 = 12.;
const COYOTE_TIME_FRAMES: u32 = 3;
const KNOCKBACK_SPEED: f64 = 15.;
const KNOCKBACK_FRAMES: usize = 60;

const BULLET_RADIUS: f64 = 0.4;
const BULLET_SPEED: f64 = 25.;
const BULLET_TILE_DAMAGE: f32 = 0.5;

pub struct PlayerState {
    pub entity: sf::hecs::Entity,
    has_doublejump: bool,
    frames_since_on_ground: u32,
    holding_jump: bool,
    // aim direction stored here
    // so that we can shoot in the previously pressed direction
    // if no direction is currently held
    aim_dir: sf::math::UnitDVec2,
    knockback_frames: usize,
}

pub struct Bullet {
    // bullets store their movement direction and move manually
    // in order to ensure they don't tunnel and only hit one thing at a time
    dir: sf::math::UnitDVec2,
}

impl PlayerState {
    pub fn spawn(game: &mut sf::Game, assets: &super::Assets) -> Self {
        // player always spawns on the bottom left of the level
        let pose = sf::PoseBuilder::new().with_position([0.5, 0.5]).build();
        // collider is currently in assets to make a mesh out of it.
        // TODO: once we have art assets it would be cleaner
        // to have the collider definition here in this file
        let coll = assets
            .player_collider
            .with_layer(crate::physics_layers::PLAYER);
        let body = game
            .physics
            .entity_set
            .insert_body(sf::Body::new_particle(PLAYER_MASS));
        let coll = game.physics.entity_set.attach_collider(body, coll);
        let mesh = assets.player_mesh;

        let entity = game.world.spawn((pose, coll, body, mesh));

        Self {
            entity,
            has_doublejump: true,
            frames_since_on_ground: 0,
            holding_jump: false,
            aim_dir: sf::math::UnitDVec2::unit_x(),
            knockback_frames: 0,
        }
    }

    pub fn tick(&mut self, game: &mut sf::Game, assets: &super::Assets) {
        // gather tiles to break into a buffer and apply at the end
        // so that we don't need nested hecs queries
        let mut tiles_touched: Vec<sf::hecs::Entity> = Vec::new();

        // read controls
        // TODO configurable keys, gamepad support
        let lr_input = game.input.axis(sf::AxisQuery {
            pos_btn: sf::Key::ArrowRight.into(),
            neg_btn: sf::Key::ArrowLeft.into(),
        });
        let tb_input = game.input.axis(sf::AxisQuery {
            pos_btn: sf::Key::ArrowUp.into(),
            neg_btn: sf::Key::ArrowDown.into(),
        });
        let jump_btn = sf::ButtonQuery::kb(sf::Key::ShiftLeft);
        let jump_input = game.input.button(jump_btn);
        let jump_released = game.input.button(jump_btn.released());
        let shoot_input = game.input.button(sf::ButtonQuery::kb(sf::Key::KeyZ));

        let Ok((&pose, &coll_key)) = game
            .world
            .query_one_mut::<(&sf::Pose, &sf::ColliderKey)>(self.entity)
        else {
            return;
        };

        // check for being on the ground and also begin destroy blocks touched
        let mut is_on_ground = false;
        let mut knockback_vel: Option<sf::DVec2> = None;
        for cont in game.physics.contacts_for_collider(coll_key) {
            if let Some(ent) = game.hecs_sync.get_collider_entity(cont.colliders[1]) {
                if let Ok((_, enemy_pose)) = game.world.query_one_mut::<(&Enemy, &sf::Pose)>(ent) {
                    knockback_vel = Some(if pose.translation.x < enemy_pose.translation.x {
                        sf::DVec2::new(-KNOCKBACK_SPEED, 0.)
                    } else {
                        sf::DVec2::new(KNOCKBACK_SPEED, 0.)
                    });

                    game.world.despawn(ent).ok();
                }
            }

            if cont.normal.y < -0.9 {
                is_on_ground = true;

                if let Some(ent) = game.hecs_sync.get_collider_entity(cont.colliders[1]) {
                    tiles_touched.push(ent);
                }
            }
        }

        let Ok((pose, body_key, mesh)) = game
            .world
            .query_one_mut::<(&sf::Pose, &sf::BodyKey, &mut sf::MeshId)>(self.entity)
        else {
            return;
        };

        if is_on_ground {
            self.has_doublejump = true;
            self.frames_since_on_ground = 0;
        } else {
            self.frames_since_on_ground += 1;
        }
        let is_coyote_time = self.frames_since_on_ground < COYOTE_TIME_FRAMES;

        // also shoot a spherecast down to check for oneway platforms
        if let Some(hit_below) = game.physics.spherecast(
            COLLIDER_WIDTH,
            sf::Ray {
                // getting the y coordinate right here
                // so that we don't get stuck in the block
                // and also won't fall through it is pretty fiddly,
                // would be nice to have a better solution for this
                // sf note: we could probably provide something like this out of the box,
                // like a collider that only resolves collisions in a specific direction
                start: sf::DVec2::new(pose.translation.x as f64, pose.translation.y as f64 + 0.3),
                dir: sf::math::UnitDVec2::new_unchecked(sf::DVec2::new(0., -1.)),
            },
            5.,
        ) {
            if let Some(coll) = game.physics.entity_set.get_collider_mut(hit_below.collider) {
                if coll.layer == crate::physics_layers::ONEWAY_INACTIVE {
                    coll.layer = crate::physics_layers::ONEWAY_ACTIVE;
                }
            }
        }

        let body = game
            .physics
            .entity_set
            .get_body_mut(*body_key)
            .expect("Player body disappeared unexpectedly");

        // controls

        if let Some(vel) = knockback_vel {
            body.velocity.linear = vel;
            self.knockback_frames = KNOCKBACK_FRAMES;
            self.has_doublejump = true;
        }

        if self.knockback_frames > 0 {
            self.knockback_frames -= 1;
        } else {
            body.velocity.linear.x = lr_input * MAX_XSPEED;

            // jump
            if jump_input && (is_coyote_time || self.has_doublejump) {
                body.velocity.linear.y = JUMP_YSPEED;
                self.holding_jump = true;
                if !is_coyote_time {
                    self.has_doublejump = false;
                }
            }
            // cut jump short when button released
            if self.holding_jump && jump_released {
                self.holding_jump = false;
                if body.velocity.linear.y > 0. {
                    body.velocity.linear.y *= 0.25;
                }
            }
        }

        // change the mesh depending on whether double jump is spent
        *mesh = if self.has_doublejump && self.knockback_frames == 0 {
            assets.player_mesh
        } else {
            assets.player_mesh_doublejumped
        };

        // shoot/aim

        if lr_input != 0. || tb_input != 0. {
            self.aim_dir = sf::math::UnitDVec2::new_normalize(sf::DVec2::new(lr_input, tb_input));
        }
        if shoot_input {
            let pose = sf::PoseBuilder::new()
                .with_position(pose.translation.xy())
                .build();
            let body = sf::Body::new_kinematic();
            let body = game.physics.entity_set.insert_body(body);
            let mesh = assets.bullet_mesh;
            let bullet = Bullet { dir: self.aim_dir };

            game.world.spawn((pose, body, mesh, bullet));
        }

        // break tiles walked on

        for ent in tiles_touched {
            if let Ok(mut tile) = game.world.get::<&mut BreakableTile>(ent) {
                tile.is_breaking = true;
            }
        }
    }

    pub fn move_camera(&self, game: &mut sf::Game, camera: &mut sf::Camera) {
        let Ok((pose,)) = game.world.query_one_mut::<(&sf::Pose,)>(self.entity) else {
            return;
        };

        if pose.translation.y > camera.pose.translation.y {
            camera.pose.translation.y = pose.translation.y;
        }
    }
}

/// Check for bullets colliding with tiles and set the tiles to break.
pub fn handle_bullets(game: &mut sf::Game, camera: &sf::Camera) {
    // gather hits first so that we don't need tricky nested query shenanigans
    let mut hits: Vec<(sf::hecs::Entity, sf::hecs::Entity)> = Vec::new();
    // also destroy bullets that have left the area visible on camera
    let mut off_cameras: Vec<sf::hecs::Entity> = Vec::new();
    for (bullet_ent, (bullet, pose)) in game.world.query_mut::<(&Bullet, &mut sf::Pose)>() {
        // check for the next thing hit by doing a spherecast
        // so that we don't end up hitting multiple things at the same time.
        // sf note: in some cases we'll need to filter cast results by collision layer.
        // it's not essential here though because the player is the only thing that shouldn't be hit
        let next_hit = game.physics.spherecast(
            BULLET_RADIUS,
            sf::Ray {
                start: sf::DVec2::new(pose.translation.x as f64, pose.translation.y as f64),
                dir: bullet.dir,
            },
            BULLET_SPEED * game.dt_fixed,
        );
        if let Some(ent) = next_hit.and_then(|hit| game.hecs_sync.get_collider_entity(hit.collider))
        {
            hits.push((bullet_ent, ent));
        }

        let step = BULLET_SPEED * game.dt_fixed * *bullet.dir;
        pose.translation.x += step.x as f32;
        pose.translation.y += step.y as f32;

        if camera
            .point_world_to_screen(pose.translation.xy())
            .is_none()
        {
            off_cameras.push(bullet_ent);
        }
    }

    for (bullet, other) in hits {
        if let Ok((tile,)) = game.world.query_one_mut::<(&mut BreakableTile,)>(other) {
            tile.is_breaking = true;
            tile.time_to_break -= BULLET_TILE_DAMAGE;
            if tile.blocks_bullets {
                game.world.despawn(bullet).ok();
            }
        } else if let Ok(true) = game.world.satisfies::<(&mut Enemy,)>(other) {
            // hitting an enemy destroys both the enemy and the bullet
            // (maybe eventually there could be enemies with HP but not yet)
            game.world.despawn(other).ok();
            game.world.despawn(bullet).ok();
        }
    }

    for bullet in off_cameras {
        game.world.despawn(bullet).ok();
    }
}
