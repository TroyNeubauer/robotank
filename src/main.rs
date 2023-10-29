use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod tank;
pub use tank::*;

mod bullet;
pub use bullet::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            TanksPlugin,
        ))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default().with_physics_scale(1.0),
            RapierDebugRenderPlugin::default(),
        ))
        .run();
}
