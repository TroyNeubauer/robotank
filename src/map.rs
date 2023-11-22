use array2d::Array2D;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};
use bevy_rapier2d::prelude::*;
use rand::{Rng, SeedableRng};

#[derive(Clone, Bundle)]
pub struct MapBundle {
    collider: Collider,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    map: Map,
}

#[derive(Clone, Component, Debug)]
pub struct Map;

impl MapBundle {
    pub fn new_empty(
        materials: &Res<crate::Materials>,
        meshes: &mut ResMut<Assets<Mesh>>,
        size: IVec2,
        world_offset: Vec2,
    ) -> Self {
        Self::new_from_tiles(materials, meshes, MapTiles::new_empty(size), world_offset)
    }

    pub fn new_from_tiles(
        materials: &Res<crate::Materials>,
        meshes: &mut ResMut<Assets<Mesh>>,
        tiles: MapTiles,
        world_offset: Vec2,
    ) -> Self {
        let mut triangle = Mesh::new(PrimitiveTopology::TriangleList);

        let mut verticies = vec![];
        let mut indices = vec![];
        let mut shapes: Vec<(Vect, Rot, Collider)> = vec![];

        let mut add_square = |pos: Vec2| {
            let i = verticies.len() as u32;

            verticies.push([pos.x + 1.0, pos.y + 1.0, 0.0]);
            verticies.push([pos.x + 0.0, pos.y + 1.0, 0.0]);
            verticies.push([pos.x + 1.0, pos.y + 0.0, 0.0]);
            verticies.push([pos.x + 0.0, pos.y + 0.0, 0.0]);

            indices.push(i + 0);
            indices.push(i + 1);
            indices.push(i + 2);

            indices.push(i + 2);
            indices.push(i + 1);
            indices.push(i + 3);

            shapes.push((pos + Vec2::new(0.5, 0.5), 0.0, Collider::cuboid(0.5, 0.5)));
        };

        for ((y, x), tile) in tiles.0.enumerate_row_major() {
            match tile {
                Tile::Air => {}
                Tile::Wall => {
                    add_square(Vec2::new(x as f32, y as f32));
                }
            }
        }

        triangle.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies.clone());
        /*triangle.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
        );*/

        triangle.set_indices(Some(Indices::U32(indices.clone())));

        Self {
            collider: Collider::compound(shapes),
            mesh: MaterialMesh2dBundle {
                mesh: meshes.add(triangle).into(),
                transform: Transform::from_xyz(world_offset.x, world_offset.y, 0.0),
                material: materials.wall_material.clone(),
                ..Default::default()
            },
            map: Map,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Tile {
    Air,
    Wall,
}

pub struct MapTiles(Array2D<Tile>);

impl MapTiles {
    pub fn new_empty(size: IVec2) -> Self {
        let mut map = Array2D::filled_with(Tile::Air, size.y as usize, size.x as usize);
        for i in 0..size.x as usize {
            map[(0, i)] = Tile::Wall;
            map[(size.y as usize - 1, i)] = Tile::Wall;
        }
        for i in 0..size.y as usize {
            map[(i, 0)] = Tile::Wall;
            map[(i, size.x as usize - 1)] = Tile::Wall;
        }
        Self(map)
    }

    /// Randomly generates a map based on `seed`.
    /// Fullness determines how "full" the map should be from 0..1
    pub fn gen_v1(size: IVec2, fullness: f32, seed: u64) -> MapTiles {
        let mut map = Self::new_empty(size);
        let mut rng = rand_chacha::ChaChaRng::seed_from_u64(seed);

        const MAX_WALL_LENGTH: usize = 6;
        let wanted_filled_tiles = size.x as f32 * size.y as f32 * fullness / 2.0;

        // the number of walls we can spawn
        let spawnable_walls = (wanted_filled_tiles / MAX_WALL_LENGTH as f32).round() as usize;
        dbg!(wanted_filled_tiles, spawnable_walls);

        let spawn_tries = 1000;
        'spawn_loop: for i in 0..spawnable_walls {
            'try_spawn: for _ in 0..spawn_tries {
                let len = rng.gen_range(1..=MAX_WALL_LENGTH);
                let dir = match rng.gen_range(0..4) {
                    0 => IVec2::new(1, 0),
                    1 => IVec2::new(-1, 0),
                    2 => IVec2::new(0, 1),
                    3 => IVec2::new(0, -1),
                    _ => unreachable!(),
                };
                let x = rng.gen_range(0..size.x);
                let y = rng.gen_range(0..size.y);
                let pos = IVec2::new(x, y);

                for v in 0..len {
                    let p = pos + dir * IVec2::new(v as i32, v as i32);

                    let Some(t) = map.try_get(p) else {
                        continue 'try_spawn;
                    };

                    if *t != Tile::Air {
                        continue 'try_spawn;
                    }
                }

                for v in 0..len {
                    let p = pos + dir * IVec2::new(v as i32, v as i32);

                    let Some(t) = map.try_get_mut(p) else {
                        continue 'try_spawn;
                    };
                    *t = Tile::Wall;
                }
                continue 'spawn_loop;
            }

            println!("Failed to add wall {i}/{spawnable_walls} after {spawn_tries}");
            break;
        }

        map
    }

    pub fn try_get(&self, p: IVec2) -> Option<&Tile> {
        if p.x < 0 || p.x >= self.0.column_len() as i32 {
            return None;
        }
        if p.y < 0 || p.y >= self.0.row_len() as i32 {
            return None;
        }

        Some(&self.0[(p.x as usize, p.y as usize)])
    }

    pub fn try_get_mut(&mut self, p: IVec2) -> Option<&mut Tile> {
        if p.x < 0 || p.x >= self.0.column_len() as i32 {
            return None;
        }
        if p.y < 0 || p.y >= self.0.column_len() as i32 {
            return None;
        }

        Some(&mut self.0[(p.x as usize, p.y as usize)])
    }

    pub fn astar(&self, start: IVec2, goal: IVec2) -> Result<AStarPath, ()> {
        let success = |c: &IVec2| c == &goal;
        let successors = |c: &IVec2| {
            dbg!(c);
            let mut v = smallvec::SmallVec::<[(IVec2, usize); 8]>::new();
            let points = [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y];
            for p in points {
                let pos = p + *c;
                if let Some(t) = self.try_get(pos) {
                    if t == &Tile::Air {
                        v.push((pos, 1));
                    }
                }
            }
            v.into_iter()
        };
        let heuristic = |c: &IVec2| (c.as_vec2() - goal.as_vec2()).length().round() as usize;
        pathfinding::directed::astar::astar(&start, successors, heuristic, success)
            .map(|(path, cost)| AStarPath { path, cost })
            .ok_or(())
    }
}

impl std::ops::Deref for MapTiles {
    type Target = Array2D<Tile>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MapTiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Component, Debug)]
pub struct AStarPath {
    pub path: Vec<IVec2>,
    pub cost: usize,
}

/*
#[derive(Clone, Bundle)]
pub struct AStarBundle {
    path: AStarPath,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl AStarBundle {
    pub fn new(
        materials: &Res<crate::Materials>,
        meshes: &mut ResMut<Assets<Mesh>>,
        path: AStarPath,
    ) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);

        let mut verticies = vec![];

        for c in &path.path {
            verticies.push([c.x as f32, c.y as f32, 0.0]);
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies.clone());

        Self {
            path,
            mesh: MaterialMesh2dBundle {
                mesh: meshes.add(mesh).into(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: materials.add(LineMaterial {
                    color: Color::GREEN,
                }),
                ..Default::default()
            },
        }
    }
}
*/
