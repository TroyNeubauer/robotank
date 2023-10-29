use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TanksPlugin,
        ))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .run();
}

pub struct TanksPlugin;

impl Plugin for TanksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (update_tank_body_input_system, update_tank_gun_input_system),
        );
    }
}

fn create_tank(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec2,
    player_controlled: bool,
) {
    let scale = 1.0 / 5.0;
    let tank = {
        let mut tank = commands.spawn(SpriteBundle {
            texture: asset_server.load("tank_base.png"),
            transform: Transform::from_xyz(position.x, position.y, 0.0)
                .with_scale(Vec3::new(scale, scale, scale)),
            ..Default::default()
        });
        tank.insert(TankBody { speed: 0.0 })
            .insert(RigidBody::Dynamic)
            .insert(Velocity {
                linvel: Vec2::new(0.0, 0.0),
                angvel: 0.0,
            })
            .insert(Collider::cuboid(200.0, 200.0))
            // The default density is 1.0, we are setting 2.0 for this example.
            .insert(ColliderMassProperties::Density(2.0))
            // XY plane is flat base, no gravity
            .insert(GravityScale(0.0))
            .insert(Damping {
                linear_damping: 1.5,
                angular_damping: 5.0,
            });

        if player_controlled {
            tank.insert(PlayerControlled);
        }
        tank.id()
    };

    let gun = {
        let mut gun = commands.spawn(SpriteBundle {
            texture: asset_server.load("tank_gun.png"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        });
        gun.insert(TankGun);

        if player_controlled {
            gun.insert(PlayerControlled);
        }
        gun.id()
    };

    commands.entity(tank).push_children(&[gun]);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    create_tank(&mut commands, &asset_server, Vec2::new(200.0, -5.0), true);
    create_tank(&mut commands, &asset_server, Vec2::new(-100.0, 50.0), false);
}

/*
fn apply_velocity(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, vel) in &mut q {
        transform.translation.x += vel.0.x * time.delta_seconds();
        transform.translation.y += vel.0.y * time.delta_seconds();
    }
}
*/

/// Input actions to tank, produced by user input or the AI
struct TankBodyInput {
    /// (0..1) strength of forward action
    forward: f32,
    /// (0..1) strength of backward action
    backward: f32,
    /// (0..1) rotation speed, (negative == counter clockwise)
    rotate: f32,
}

struct TankGunInput {
    /// Desired gun angle (radians)
    gun_angle: f32,
}

impl TankBodyInput {
    pub fn new(forward: f32, backward: f32, rotate: f32) -> Self {
        Self {
            forward: forward.clamp(0.0, 1.0),
            backward: backward.clamp(0.0, 1.0),
            rotate: rotate.clamp(-1.0, 1.0),
        }
    }
}

impl TankGunInput {
    pub fn new(gun_angle: f32) -> Self {
        Self {
            gun_angle: gun_angle.rem_euclid(2.0 * PI),
        }
    }
}

fn get_rotz(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::XYZ).2
}

fn update_tank_gun_input(
    time: &Res<Time>,
    input: &TankGunInput,
    local: &mut Transform,
    global: &GlobalTransform,
    _gun: &TankGun,
) {
    let transform = global.compute_transform();

    let desired = Quat::from_rotation_z(input.gun_angle);
    let error2 = transform.rotation.angle_between(desired);

    // How many radians should we step closer to `desired` this update
    let scalar_step = (220.0 * time.delta_seconds()).to_radians();

    // interpolation factor between our current rotation and desired
    let f = (scalar_step / error2).clamp(0.0, 1.0);
    // the global rotation we want this update
    let rotated = transform.rotation.lerp(desired, f);

    // We can only modify `local`, so find the delta between transform and rotated to apply locally
    let step = rotated * transform.rotation.inverse();

    // apply rotation
    local.rotate(step);
}

fn update_tank_gun_input_system(
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(
        &mut Transform,
        &GlobalTransform,
        &TankGun,
        &PlayerControlled,
    )>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();

    let cursor_pos = window.single().cursor_position();
    for (mut local, global, gun, _) in &mut q {
        let tank_pos = global.translation();
        let gun_angle = cursor_pos
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| {
                let delta = ray.origin.truncate() - tank_pos.truncate();
                let r = f32::atan2(delta.y, delta.x);
                r
            })
            // target current rotation if mouse outside screen
            .unwrap_or_else(|| get_rotz(&global.compute_transform()));

        let input = TankGunInput::new(gun_angle);

        update_tank_gun_input(&time, &input, &mut local, global, gun)
    }
}

fn update_tank_body_input(
    time: &Res<Time>,
    input: &TankBodyInput,
    transform: &mut Transform,
    vel: &mut Velocity,
    body: &mut TankBody,
) {
    let mut accerlating = false;
    let rotate_speed = 140.0;

    if input.rotate != 0.0 {
        transform.rotate_z((input.rotate * rotate_speed * time.delta_seconds()).to_radians());
    }

    if input.forward != 0.0 {
        body.speed += input.forward * 300.0 * time.delta_seconds();
        accerlating = true;
    }

    if input.backward != 0.0 {
        body.speed -= input.backward * 300.0 * time.delta_seconds();
        accerlating = true;
    }
    body.speed = body.speed.clamp(-100.0, 100.0);

    if !accerlating {
        let decrease = 150.0 * time.delta_seconds();
        let decrease = decrease.clamp(0.0, body.speed.abs());
        body.speed -= body.speed.signum() * decrease;
    }

    let rotation = get_rotz(&transform);

    vel.linvel = Vec2::from_angle(rotation) * body.speed;
}

fn update_tank_body_input_system(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut q: Query<(
        &mut Transform,
        &mut Velocity,
        &mut TankBody,
        &PlayerControlled,
    )>,
) {
    for (mut transform, mut vel, mut body, _) in &mut q {
        let forward = keys.pressed(KeyCode::Up).then(|| 1.0).unwrap_or(0.0);
        let backward = keys.pressed(KeyCode::Down).then(|| 1.0).unwrap_or(0.0);

        let left_rotate = keys.pressed(KeyCode::Left).then(|| 1.0).unwrap_or(0.0);
        let right_rotate = keys.pressed(KeyCode::Right).then(|| -1.0).unwrap_or(0.0);

        let rotate = left_rotate + right_rotate;

        let input = TankBodyInput::new(forward, backward, rotate);
        update_tank_body_input(&time, &input, &mut transform, &mut vel, &mut body);
    }
}

#[derive(Component)]
struct TankGun;

#[derive(Component)]
struct TankBody {
    speed: f32,
}

#[derive(Component)]
struct PlayerControlled;
