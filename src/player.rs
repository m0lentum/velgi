use starframe as sf;

const PLAYER_MASS: f64 = 1.;
const MAX_XSPEED: f64 = 5.;
const JUMP_YSPEED: f64 = 10.;

struct PlayerState {
    has_doublejump: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            has_doublejump: true,
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

pub fn tick(game: &mut sf::Game) {
    for (_, (state, body_key)) in game.world.query_mut::<(&mut PlayerState, &sf::BodyKey)>() {
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
        let jump_input = game.input.button(sf::Key::ShiftLeft.into());

        body.velocity.linear.x = lr_input * MAX_XSPEED;

        if jump_input && state.has_doublejump {
            body.velocity.linear.y = JUMP_YSPEED;
            state.has_doublejump = false;
        }

        state.has_doublejump = true;
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
