use array2d::Array2D;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};
use bevy_rapier2d::prelude::*;

#[derive(Clone, Component)]
pub struct MapBundle {
    //collider: Collider,
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
        let verticies = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let indices = vec![0, 1, 2];

        triangle.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies.clone());
        /*triangle.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
        );*/

        triangle.set_indices(Some(Indices::U32(indices)));

        let indices2d: Vec<_> = (0..(verticies.len() - 1))
            .enumerate()
            .map(|(i, _)| [i as u32, i as u32 + 1])
            .chain(Some([verticies.len() as u32 - 1, 0]))
            .collect();

        let verticies2d: Vec<_> = verticies
            .into_iter()
            .map(|v| Vec2::new(v[0], v[1]))
            .collect();

        dbg!(&indices2d, &verticies2d);

        Self {
            //collider: Collider::convex_decomposition(&verticies2d, &indices2d),
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
