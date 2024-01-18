use bevy::{
    math::cubic_splines::CubicCurve,
    prelude::*,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
    sprite::Mesh2dHandle,
};

pub struct BevyPcgSplinesPlugin;

#[derive(Component)]
pub struct PcgSpline {
    pub mesh_2d: Mesh2dHandle,
    pub curve: CubicCurve<Vec3>,
}

impl Plugin for BevyPcgSplinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<NormalsMaterial>::default())
            .add_systems(
                // TODO: generates meshes in runtime
                PostStartup,
                (Self::generate_mesh,),
            );
    }
}

impl BevyPcgSplinesPlugin {
    const RING_COUNT: usize = 128;

    fn generate_mesh(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        query: Query<&PcgSpline>,
    ) {
        for spline in query.iter() {
            let curves_iter = spline.curve.iter_positions(Self::RING_COUNT - 1);
            let velocities_iter = spline.curve.iter_velocities(Self::RING_COUNT - 1);
            let points_normals = curves_iter
                .zip(velocities_iter)
                .map(|(pos, vel)| (pos, Quat::from_rotation_arc(Vec3::Z, vel.normalize())));

            let mesh_2d_handle = &spline.mesh_2d.0;
            let mesh_2d = meshes
                .get_mut(mesh_2d_handle.id())
                .expect("Could not find mesh 2d");

            let (verts, vert_normals): (Vec<Vec3>, Vec<Vec3>) = points_normals
                .flat_map(|(point, rot)| {
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

                    positions
                        .iter()
                        .zip(normals.iter())
                        .map(move |(&pos, &norm)| {
                            let local_point = Vec3::from(pos);
                            let local_normal = Vec3::from(norm);
                            (point + rot * local_point, rot * local_normal)
                        })
                })
                .unzip();

            let stride = mesh_2d.count_vertices();
            let indices = (0..Self::RING_COUNT - 1)
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

            commands.spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::PURPLE.into()),
                ..default()
            });
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct NormalsMaterial {}

impl Material for NormalsMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/normals.wgsl".into()
    }
}
