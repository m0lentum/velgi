use starframe as sf;

pub const PLAYER: usize = 1;
pub const BULLET: usize = 2;
// one-way platforms implemented by ignoring collisions with the player
// until the player moves above them,
// then swapping them to a layer that has collision with the player
pub const ONEWAY_INACTIVE: usize = 3;
pub const ONEWAY_ACTIVE: usize = 4;
pub const SPIKE_ROLLER: usize = 5;
pub const ENEMY: usize = 6;

pub fn setup(physics: &mut sf::PhysicsWorld) {
    physics.mask_matrix.ignore(PLAYER, BULLET);
    physics.mask_matrix.ignore(PLAYER, ONEWAY_INACTIVE);
    physics.mask_matrix.ignore(ENEMY, ONEWAY_ACTIVE);
    physics.mask_matrix.ignore(ENEMY, ONEWAY_INACTIVE);
}
