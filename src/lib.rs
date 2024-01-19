use bevy::{
    gizmos,
    math::cubic_splines::CubicCurve,
    prelude::*,
    render::{
        mesh::{Indices},
        render_resource::PrimitiveTopology,
    },
    sprite::Mesh2dHandle,
};

// TODO: benchmarks
pub struct BevyPgcSplinesPlugin;

#[derive(Component)]
pub struct PgcSpline {
    pub mesh_2d: Mesh2dHandle,
    pub curve: CubicCurve<Vec3>,
    pub ring_count: usize,
}

// TODO: replace material_mesh with individual components? excluding Mesh
#[derive(Bundle)]
pub struct PgcSplineBundle<M: Material> {
    pub spline: PgcSpline,
    // Replaces the mesh within this with the generated  mesh
    pub material_mesh: MaterialMeshBundle<M>,
}

impl Plugin for BevyPgcSplinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            // TODO: cache most data
            // TODO: only run if mesh changes
            Update,
            (Self::generate_mesh,),
        );
    }
}

impl BevyPgcSplinesPlugin {
    pub fn generate_mesh(
        mut meshes: ResMut<Assets<Mesh>>,
        mut query: Query<(&PgcSpline, &mut Handle<Mesh>)>,
        mut gizmos: Gizmos,
    ) {
        for (spline, mut og_mesh) in query.iter_mut() {
            let curves_iter = spline.curve.iter_positions(spline.ring_count - 1);
            let velocities_iter = spline.curve.iter_velocities(spline.ring_count - 1);
            let points_normals = curves_iter.zip(velocities_iter).map(|(pos, vel)| {
                // TODO: switch between -Vec3::Z and Vec3::Z depending on curve
                let quat = Quat::from_rotation_arc(-Vec3::Z, vel.normalize());
                gizmos.rect(pos, quat, Vec2::splat(0.5), Color::WHITE);
                (pos, quat)
            });

            let mesh_2d_handle = &spline.mesh_2d.0;
            let mesh_2d = meshes
                .get_mut(mesh_2d_handle.id())
                .expect("Could not find mesh 2d");

            let positions = mesh_2d
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .expect("Mesh missing positions")
                .as_float3()
                .expect("Expected ATTRIBUTE_POSITION to be Float32x3");
            let normals = mesh_2d
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .expect("Mesh missing normals")
                .as_float3()
                .expect("Expected ATTRIBUTE_NORMAL to be Float32x3");

            // for (spline_pos, spline_rot) in points_normals {
            //     for (local_point, local_normal) in positions.iter().zip(normals.iter()) {
            //         let local_point = Vec3::from_array(*local_point);
            //         let local_normal = Vec3::from_array(*local_normal);
            //         // gizmos.ray(spline_pos, spline_rot * local_point, Color::WHITE);
            //         // gizmos.ray(spline_pos + spline_rot * local_point, local_normal, Color::RED);
            //     }
            // }

            let (verts, vert_normals): (Vec<Vec3>, Vec<Vec3>) = points_normals
                .flat_map(|(point, rot)| {
                    {
                        positions
                            .iter()
                            .zip(normals.iter())
                            .map(move |(&pos, &norm)| {
                                let local_point = Vec3::from(pos);
                                let local_normal = Vec3::from(norm);
                                (point + (rot * local_point), rot * local_normal)
                            })
                    }
                })
                .unzip();

            // TODO: allow for shapes with holes (ie. donut)
            let stride = mesh_2d.count_vertices();
            let indices = (0..spline.ring_count - 1)
                .flat_map(|ring| {
                    (0..stride).flat_map(move |i| {
                        let next_i = (i + 1) % stride;
                        [
                            (ring * stride + i) as u32,
                            ((ring + 1) * stride + i) as u32,
                            ((ring + 1) * stride + next_i) as u32,
                            (ring * stride + i) as u32,
                            ((ring + 1) * stride + next_i) as u32,
                            (ring * stride + next_i) as u32,
                        ]
                    })
                })
                .collect::<Vec<_>>();

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vert_normals);
            mesh.set_indices(Some(Indices::U32(indices)));
            *og_mesh = meshes.add(mesh);
        }
    }
}
