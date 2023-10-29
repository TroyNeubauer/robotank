use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct Bullet {
    shooter: Entity,
}

#[derive(Resource)]
pub struct BulletAsset(pub Handle<Image>);

pub fn spawn_bullet(
    commands: &mut Commands,
    tank: Entity,
    bullet: &Res<BulletAsset>,
    size: Vec2,
    pos: Vec2,
    vel: Vec2,
) {
    let mut bullet = commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(size),
            ..Default::default()
        },
        texture: bullet.0.clone(),
        transform: Transform::from_xyz(pos.x, pos.y, 0.0),
        ..Default::default()
    });

    bullet
        .insert(RigidBody::Dynamic)
        .insert(Velocity {
            linvel: vel,
            angvel: 0.0,
        })
        .insert(Collider::cuboid(size.x / 2.0, size.y / 2.0))
        .insert(ColliderMassProperties::Density(3.0))
        // XY plane is flat base, no gravity
        .insert(GravityScale(0.0))
        .insert(Bullet { shooter: tank });
}
