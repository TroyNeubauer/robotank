use array2d::Array2D;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};
use bevy_rapier2d::prelude::*;

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
    ) -> Self {
        Self::new_from_mesh(materials, meshes, MapTiles::new_empty(size))
    }

    fn new_from_mesh(
        materials: &Res<crate::Materials>,
        meshes: &mut ResMut<Assets<Mesh>>,
        tiles: MapTiles,
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

        for x in (-1)..tiles.0.column_len() as isize {
            add_square(Vec2::new(x as f32, -1.0));
            add_square(Vec2::new(x as f32 + 1.0, tiles.0.row_len() as f32));
        }

        for y in (-1)..tiles.0.row_len() as isize {
            add_square(Vec2::new(-1.0, y as f32 + 1.0));
            add_square(Vec2::new(tiles.0.column_len() as f32, y as f32));
        }

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
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: materials.wall_material.clone(),
                ..Default::default()
            },
            map: Map,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
enum Tile {
    Air,
    Wall,
}

struct MapTiles(Array2D<Tile>);

impl MapTiles {
    pub fn new_empty(size: IVec2) -> Self {
        Self(Array2D::filled_with(
            Tile::Air,
            size.y as usize,
            size.x as usize,
        ))
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
