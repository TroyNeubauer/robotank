use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use std::f32::consts::{PI, SQRT_2};

#[derive(Clone, Component, Debug)]
pub struct TankGun {
    timer: Timer,
    ammo: usize,
    max_ammo: usize,
}

impl TankGun {
    pub fn new(max_ammo: usize) -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            ammo: max_ammo,
            max_ammo,
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct TankBody {
    // Tanks can only move forward or backward, this is the current speed, positive for forward
    pub speed: f32,
    pub name: String,
}

/// This entity should obtain its input from keyboard / mouse
#[derive(Clone, Component, Debug)]
pub struct PlayerControlled;

/// Input actions to tank, produced by user input or the AI
pub struct TankBodyInput {
    /// (0..1) strength of forward action
    forward: f32,
    /// (0..1) strength of backward action
    backward: f32,
    /// (0..1) rotation speed, (negative == counter clockwise)
    rotate: f32,
}

pub struct TankGunInput {
    /// Desired gun angle (radians)
    gun_angle: f32,
    /// True if shooting the gun is desired this update
    shoot: bool,
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
    pub fn new(gun_angle: f32, shoot: bool) -> Self {
        Self {
            gun_angle: gun_angle.rem_euclid(2.0 * PI),
            shoot,
        }
    }
}

pub fn get_rotz(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::XYZ).2
}

pub fn spawn_tank(
    commands: &mut Commands,
    materials: &Res<crate::Materials>,
    position: Vec2,
    name: String,
    player_controlled: bool,
) -> Entity {
    let tank = {
        let size = Vec2::new(1.0, 1.0);
        let mut tank = commands.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..Default::default()
            },
            texture: materials.tank_base.clone(),
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            ..Default::default()
        });

        tank.insert(TankBody { speed: 0.0, name })
            .insert(RigidBody::Dynamic)
            .insert(Velocity {
                linvel: Vec2::new(0.0, 0.0),
                angvel: 0.0,
            })
            .insert(Collider::cuboid(size.x / 2.0, size.y / 2.0))
            .insert(ColliderMassProperties::Density(20.0))
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
            sprite: Sprite {
                custom_size: Some(Vec2::new(619.0 / 300.0, 188.0 / 300.0)),
                ..Default::default()
            },
            texture: materials.tank_gun.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        });
        gun.insert(TankGun::new(5));

        if player_controlled {
            gun.insert(PlayerControlled);
        }
        gun.id()
    };

    commands.entity(tank).push_children(&[gun]);

    tank
}

fn reload_tank_guns(time: Res<Time>, mut q: Query<&mut TankGun>) {
    for mut gun in &mut q {
        if gun.timer.tick(time.delta()).just_finished() {
            gun.ammo = (gun.ammo + 1).clamp(0, gun.max_ammo);
        }
    }
}

const MAX_TANK_SPEED: f32 = 2.0;
const TANK_ACCLERATION: f32 = 6.0;
const BULLET_SHOOT_SPEED: f32 = 18.0;
const TANK_BRAKING: f32 = 4.0;
const TANK_ROTATE_RATE_DEGS: f32 = 140.0f32;
const GUN_ROTATE_RATE_DEGS: f32 = 220.0f32;

fn update_tank_gun_input(
    commands: &mut Commands,
    materials: &Res<crate::Materials>,
    time: &Res<Time>,
    input: &TankGunInput,
    local: &mut Transform,
    global: &GlobalTransform,
    gun: &mut TankGun,
    tank_entity: Entity,
    q_collider: &Query<&Collider>,
) {
    let transform = global.compute_transform();

    let desired = Quat::from_rotation_z(input.gun_angle);
    let gun_angle = get_rotz(&transform);
    let angular_error = transform.rotation.angle_between(desired);
    let angular_error = if angular_error.is_nan() {
        0.0
    } else {
        angular_error
    };

    // How many radians should we step closer to `desired` this update
    let scalar_step = GUN_ROTATE_RATE_DEGS.to_radians() * time.delta_seconds();

    // interpolation factor between our current rotation and desired
    let f = (scalar_step / angular_error).clamp(0.0, 1.0);
    // the global rotation we want this update
    let rotated = transform.rotation.lerp(desired, f);

    // We can only modify `local`, so find the delta between transform and rotated to apply locally
    let step = rotated * transform.rotation.inverse();

    // apply rotation
    if !step.is_nan() {
        local.rotate(step);
    }

    if input.shoot {
        if gun.ammo > 0 {
            gun.ammo -= 1;
            let collider = q_collider
                .get(tank_entity)
                .expect("Parent of gun entity must have collider");
            let tank_pos = transform.translation.truncate();

            let tank_extents = collider
                .as_cuboid()
                .expect("Only cubiod colliders are allowed for tanks")
                .raw
                .half_extents;

            let bullet_size = Vec2::new(0.2, 0.2);
            // "radius" is the radius of the circle that inscribes the bounding box
            // (prevents the bullet from colliding with the shooting tank immediately)
            let radius = (tank_extents.x.max(tank_extents.y) + bullet_size.x) * SQRT_2;

            let bullet_velocity_unit = Vec2::from_angle(gun_angle);
            // Spawn bullet outside of the tank's hitbox
            let bullet_pos = tank_pos + bullet_velocity_unit * radius;
            let bullet_velocity = BULLET_SHOOT_SPEED * bullet_velocity_unit;

            crate::spawn_bullet(
                commands,
                materials,
                tank_entity,
                bullet_size,
                bullet_pos,
                bullet_velocity,
            );
        }
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

    if input.rotate != 0.0 {
        transform
            .rotate_z(input.rotate * TANK_ROTATE_RATE_DEGS.to_radians() * time.delta_seconds());
    }

    if input.forward != 0.0 {
        body.speed += input.forward * TANK_ACCLERATION * time.delta_seconds();
        accerlating = true;
    }

    if input.backward != 0.0 {
        body.speed -= input.backward * TANK_ACCLERATION * time.delta_seconds();
        accerlating = true;
    }
    body.speed = body.speed.clamp(-MAX_TANK_SPEED, MAX_TANK_SPEED);

    // brake if no forward or backward inputs are given
    if !accerlating {
        let decrease = TANK_BRAKING * time.delta_seconds();
        let decrease = decrease.clamp(0.0, body.speed.abs());
        body.speed -= body.speed.signum() * decrease;
    }

    let rotation = get_rotz(&transform);

    vel.linvel = Vec2::from_angle(rotation) * body.speed;
}

fn update_tank_body_input_system(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut q_tank: Query<(
        &mut Transform,
        &mut Velocity,
        &mut TankBody,
        With<PlayerControlled>,
    )>,
) {
    for (mut transform, mut vel, mut body, _) in &mut q_tank {
        let forward = keys.pressed(KeyCode::Up).then(|| 1.0).unwrap_or(0.0);
        let backward = keys.pressed(KeyCode::Down).then(|| 1.0).unwrap_or(0.0);

        let left_rotate = keys.pressed(KeyCode::Left).then(|| 1.0).unwrap_or(0.0);
        let right_rotate = keys.pressed(KeyCode::Right).then(|| -1.0).unwrap_or(0.0);

        let rotate = left_rotate + right_rotate;

        let input = TankBodyInput::new(forward, backward, rotate);
        update_tank_body_input(&time, &input, &mut transform, &mut vel, &mut body);
    }
}

fn update_tank_gun_input_system(
    mut commands: Commands,
    materials: Res<crate::Materials>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut q_gun: Query<(
        &mut Transform,
        &GlobalTransform,
        &mut TankGun,
        &Parent,
        With<PlayerControlled>,
    )>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_collider: Query<&Collider>,
) {
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };

    let Ok(cursor_pos) = window.get_single().map(|w| w.cursor_position()) else {
        return;
    };
    for (mut local, global, mut gun, parent, _) in &mut q_gun {
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

        let shoot = buttons.just_pressed(MouseButton::Left);
        let input = TankGunInput::new(gun_angle, shoot);

        update_tank_gun_input(
            &mut commands,
            &materials,
            &time,
            &input,
            &mut local,
            global,
            &mut gun,
            parent.get(),
            &q_collider,
        )
    }
}

pub fn init_tank_systems(app: &mut App) {
    app.add_systems(Update, update_tank_body_input_system);
    app.add_systems(Update, update_tank_gun_input_system);
    app.add_systems(Update, reload_tank_guns);
}
