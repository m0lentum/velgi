use starframe as sf;

pub const PLAYER: usize = 1;
pub const BULLET: usize = 2;

pub fn setup(physics: &mut sf::PhysicsWorld) {
    physics.mask_matrix.ignore(PLAYER, BULLET);
}
