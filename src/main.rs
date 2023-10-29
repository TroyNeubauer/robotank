use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod tank;
pub use tank::*;

mod bullet;
pub use bullet::*;

mod game;
pub use game::*;

fn main() {
    let debug_physics = false;
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, TanksPlugin))
        .add_plugins((RapierPhysicsPlugin::<NoUserData>::default().with_physics_scale(1.0),))
        .add_systems(PostUpdate, game::display_events);

    if debug_physics {
        app.add_plugins(RapierDebugRenderPlugin::default());
    }

    app.run();
}
