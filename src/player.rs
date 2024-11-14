use starframe as sf;

const PLAYER_MASS: f64 = 1.;
const MAX_XSPEED: f64 = 7.;
const JUMP_YSPEED: f64 = 12.;
const COYOTE_TIME_FRAMES: u32 = 3;

struct PlayerState {
    has_doublejump: bool,
    frames_since_on_ground: u32,
    holding_jump: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            has_doublejump: true,
            frames_since_on_ground: 0,
            holding_jump: false,
        }
    }
}

pub fn spawn(game: &mut sf::Game, assets: &super::Assets) {
    let state = PlayerState::default();
    // player always spawns on the bottom left of the level
    let pose = sf::PoseBuilder::new().with_position([0.5, 0.5]).build();
    // collider is currently in assets to make a mesh out of it.
    // TODO: once we have art assets it would be cleaner
    // to have the collider definition here in this file
    let coll = assets.player_collider;
    let body = game
        .physics
        .entity_set
        .insert_body(sf::Body::new_particle(PLAYER_MASS));
    let coll = game.physics.entity_set.attach_collider(body, coll);
    let mesh = assets.player_mesh;

    game.world.spawn((state, pose, coll, body, mesh));
}

pub fn tick(game: &mut sf::Game, assets: &super::Assets) {
    for (_, (state, body_key, coll_key, mesh)) in game.world.query_mut::<(
        &mut PlayerState,
        &sf::BodyKey,
        &sf::ColliderKey,
        &mut sf::MeshId,
    )>() {
        let is_on_ground = game
            .physics
            .contacts_for_collider(*coll_key)
            .any(|c| c.normal.y < -0.9);
        if is_on_ground {
            state.has_doublejump = true;
            state.frames_since_on_ground = 0;
        } else {
            state.frames_since_on_ground += 1;
        }
        let is_coyote_time = state.frames_since_on_ground < COYOTE_TIME_FRAMES;

        let body = game
            .physics
            .entity_set
            .get_body_mut(*body_key)
            .expect("Player body disappeared unexpectedly");

        // controls

        // TODO configurable keys, gamepad support
        let lr_input = game.input.axis(sf::AxisQuery {
            pos_btn: sf::Key::ArrowRight.into(),
            neg_btn: sf::Key::ArrowLeft.into(),
        });
        let jump_btn = sf::ButtonQuery::kb(sf::Key::ShiftLeft);
        let jump_input = game.input.button(jump_btn);
        let jump_released = game.input.button(jump_btn.released());

        body.velocity.linear.x = lr_input * MAX_XSPEED;

        // jump
        if jump_input && (is_coyote_time || state.has_doublejump) {
            body.velocity.linear.y = JUMP_YSPEED;
            state.holding_jump = true;
            if !is_coyote_time {
                state.has_doublejump = false;
            }
        }
        // cut jump short when button released
        if state.holding_jump && jump_released {
            state.holding_jump = false;
            if body.velocity.linear.y > 0. {
                body.velocity.linear.y *= 0.25;
            }
        }

        // change the mesh depending on whether double jump is spent
        *mesh = if state.has_doublejump {
            assets.player_mesh
        } else {
            assets.player_mesh_doublejumped
        };
    }
}

pub fn move_camera(game: &mut sf::Game, camera: &mut sf::Camera) {
    let Some((_, (_state, pose))) = game
        .world
        .query_mut::<(&PlayerState, &sf::Pose)>()
        .into_iter()
        .next()
    else {
        return;
    };

    if pose.translation.y > camera.pose.translation.y {
        camera.pose.translation.y = pose.translation.y;
    }
}
