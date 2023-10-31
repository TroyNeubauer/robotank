use std::time::Duration;

use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*, render::camera::ScalingMode};
use bevy_rapier2d::prelude::*;

#[derive(Resource)]
pub struct Materials {
    pub bullet: Handle<Image>,
    pub tank_base: Handle<Image>,
    pub tank_gun: Handle<Image>,
    pub explosion: Handle<TextureAtlas>,
    pub wall_material: Handle<ColorMaterial>,
}

#[derive(Component)]
struct AnimatedSpriteSheet {
    first: usize,
    last: usize,
    timer: Timer,
    despawn_on_end: bool,
}

pub struct TanksPlugin;

impl Plugin for TanksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_materials);
        app.add_systems(PostStartup, start_game);
        app.add_systems(Update, animate_sprite);
        app.add_systems(PostUpdate, sync_player_camera);

        crate::init_tank_systems(app);
    }
}

fn load_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let explosion_handle = asset_server.load("explosion_sheet.png");
    let explosion_atlas =
        TextureAtlas::from_grid(explosion_handle, Vec2::new(256., 256.), 8, 6, None, None);
    let explosion = texture_atlases.add(explosion_atlas);

    let wall_material = materials.add(ColorMaterial::from(Color::PURPLE));

    commands.insert_resource(Materials {
        bullet: asset_server.load("bullet.png"),
        tank_base: asset_server.load("tank_base.png"),
        tank_gun: asset_server.load("tank_gun.png"),
        explosion,
        wall_material,
    });
}

fn start_game(mut commands: Commands, materials: Res<Materials>, mut meshes: ResMut<Assets<Mesh>>) {
    let mut projection = OrthographicProjection::default();
    projection.scaling_mode = ScalingMode::FixedVertical(20.0);
    projection.near = -1000.0;
    projection.far = 1000.0;

    let camera = commands
        .spawn(Camera2dBundle {
            projection,
            ..Default::default()
        })
        .id();

    let player = crate::spawn_tank(
        &mut commands,
        &materials,
        Vec2::new(2.0, 2.0),
        "Troy".into(),
        true,
    );

    commands.entity(player).add_child(camera);

    crate::spawn_tank(
        &mut commands,
        &materials,
        Vec2::new(-5.0, 0.0),
        "A.I".into(),
        false,
    );
    crate::spawn_tank(
        &mut commands,
        &materials,
        Vec2::new(2.0, -6.0),
        "A.I3".into(),
        false,
    );
    crate::spawn_tank(
        &mut commands,
        &materials,
        Vec2::new(-5.0, 7.0),
        "A.I2".into(),
        false,
    );

    let map = crate::MapBundle::new_empty(&materials, &mut meshes, IVec2::new(10, 10));
    commands.spawn(map);
}

pub fn sync_player_camera(
    mut camera: Query<(&Parent, &mut Transform), With<Camera>>,
    q_parent: Query<&GlobalTransform>,
) {
    let Ok((player, mut camera)) = camera.get_single_mut() else {
        return;
    };
    let Ok(p) = q_parent.get(player.get()) else {
        return;
    };

    let t = p.compute_transform();
    let r = crate::get_rotz(&t);
    dbg!(r);
    camera.rotation = Quat::from_rotation_z(-r);
}

pub fn display_events(
    mut commands: Commands,
    materials: Res<Materials>,
    mut collision_events: EventReader<CollisionEvent>,
    q_tank: Query<(&crate::TankBody, &Transform)>,
    q_bullet: Query<&crate::Bullet>,
) {
    for event in collision_events.iter() {
        println!("Received collision event: {event:?}");
        if let CollisionEvent::Started(a, b, _flags) = event {
            if let Ok(((tank, tank_transform), bullet, tank_entity, bullet_entity)) =
                query_dual_entities(*a, *b, &q_tank, &q_bullet)
            {
                if let Ok((shooter, _)) = q_tank.get(bullet.shooter) {
                    println!("{} killed {}", shooter.name, tank.name);
                }
                commands.entity(tank_entity).despawn_recursive();
                commands.entity(bullet_entity).despawn_recursive();

                spawn_explosion(
                    &mut commands,
                    &materials,
                    tank_transform.translation.truncate(),
                    Duration::from_secs_f32(1.2),
                )
            }
        }
    }
}

fn query_dual_entities<'q, I1, I2, Q1, Q2>(
    a: Entity,
    b: Entity,
    q1: &'q Query<Q1>,
    q2: &'q Query<Q2>,
) -> Result<(I1, I2, Entity, Entity), ()>
where
    Q1: ReadOnlyWorldQuery<Item<'q> = I1>,
    Q2: ReadOnlyWorldQuery<Item<'q> = I2>,
{
    if let Ok(i1) = q1.get(a) {
        if let Ok(i2) = q2.get(b) {
            return Ok((i1, i2, a, b));
        }
    }

    if let Ok(i1) = q1.get(b) {
        if let Ok(i2) = q2.get(a) {
            return Ok((i1, i2, b, a));
        }
    }

    Err(())
}

fn animate_sprite(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimatedSpriteSheet, &mut TextureAtlasSprite)>,
) {
    for (entity, mut animation, mut sprite) in &mut query {
        if animation.timer.tick(time.delta()).just_finished() {
            sprite.index = if sprite.index == animation.last - 1 {
                if animation.despawn_on_end {
                    commands.entity(entity).despawn_recursive();
                    animation.first
                } else {
                    animation.first
                }
            } else {
                sprite.index + 1
            };
        }
    }
}

fn spawn_explosion(
    commands: &mut Commands,
    materials: &Res<Materials>,
    pos: Vec2,
    length: Duration,
) {
    let frames = 6 * 8;
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: materials.explosion.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                custom_size: Some(Vec2::new(1.5, 1.5)),
                ..Default::default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0),
            ..default()
        },
        AnimatedSpriteSheet {
            first: 0,
            last: frames,
            timer: Timer::new(length / frames as u32, TimerMode::Repeating),
            despawn_on_end: true,
        },
    ));
}
