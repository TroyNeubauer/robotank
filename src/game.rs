use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*, render::camera::ScalingMode};
use bevy_rapier2d::prelude::*;

pub struct TanksPlugin;

impl Plugin for TanksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_materials);
        app.add_systems(PostStartup, start_game);
        crate::init_tank_systems(app);
    }
}

fn load_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // create explosion texture atlas
    let explosion_handle = asset_server.load("explosion_sheet.png");
    let explosion_atlas =
        TextureAtlas::from_grid(explosion_handle, Vec2::new(64., 64.), 4, 4, None, None);
    let explosion = texture_atlases.add(explosion_atlas);

    commands.insert_resource(Materials {
        bullet: asset_server.load("bullet.png"),
        tank_base: asset_server.load("tank_base.png"),
        tank_gun: asset_server.load("tank_gun.png"),
        explosion,
    });
}

fn start_game(mut commands: Commands, materials: Res<Materials>) {
    let mut projection = OrthographicProjection::default();
    projection.scaling_mode = ScalingMode::FixedVertical(20.0);
    projection.near = -1000.0;
    projection.far = 1000.0;
    commands.spawn(Camera2dBundle {
        projection,
        ..Default::default()
    });

    crate::spawn_tank(
        &mut commands,
        &materials,
        Vec2::new(2.0, 2.0),
        "Troy".into(),
        true,
    );

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
}

#[derive(Resource)]
pub struct Materials {
    pub bullet: Handle<Image>,
    pub tank_base: Handle<Image>,
    pub tank_gun: Handle<Image>,
    pub explosion: Handle<TextureAtlas>,
}

pub fn display_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    q_tank: Query<&crate::TankBody>,
    q_bullet: Query<&crate::Bullet>,
) {
    for event in collision_events.iter() {
        println!("Received collision event: {event:?}");
        if let CollisionEvent::Started(a, b, _flags) = event {
            if let Ok((tank, bullet, tank_entity, bullet_entity)) =
                query_dual_entities(*a, *b, &q_tank, &q_bullet)
            {
                if let Ok(shooter) = q_tank.get(bullet.shooter) {
                    println!("{} killed {}", shooter.name, tank.name);
                }
                commands.entity(tank_entity).despawn_recursive();
                commands.entity(bullet_entity).despawn_recursive();
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
