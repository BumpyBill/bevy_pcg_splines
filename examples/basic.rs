use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    math::{vec3, vec2},
    prelude::*,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};
use bevy_flycam::PlayerPlugin;
use bevy_pgc_splines::{BevyPgcSplinesPlugin, PgcSpline, PgcSplineBundle};

// TODO: benchmarks
// TODO: restructure project to allow for 1d extrusion to 2d and 3d extrusion to 3d
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy PGC Splines".to_string(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            MaterialPlugin::<DebugMaterial>::default(),
            BevyPgcSplinesPlugin,
            PlayerPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, (setup, setup_curve))
        .add_systems(Update, move_light)
        .run();
}

fn move_light(
    mut lights: Query<&mut Transform, (With<PointLight>, Without<Camera>)>,
    cameras: Query<&Transform, (With<Camera>, Without<PointLight>)>,
) {
    let mut light = lights.single_mut();
    let camera = cameras.single();
    light.translation = camera.translation;
}

fn setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}

fn setup_curve(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let control_points = [[
        vec3(0., 0., 0.),
        vec3(2., 0., 0.),
        vec3(4., 1., 0.),
        vec3(6., 1., 3.),
    ]];
    let spline = CubicBezier::new(control_points);
    let curve = spline.to_curve();

    commands.spawn(PgcSplineBundle {
        spline: PgcSpline {
            mesh_2d: bevy::sprite::Mesh2dHandle(meshes.add(generate_path(0.5))),
            curve,
            ring_count: 16,
        },
        material_mesh: MaterialMeshBundle {
            material: materials.add(StandardMaterial {..default()}),
            ..default()
        },
    });
}

fn generate_path(size: f32) -> Mesh {
    let positions = vec![vec3(-size, 0., 0.), vec3(size, 0., 0.)];
    let normals = vec![vec3(0., 1., 0.), vec3(0., 1., 0.)];
    let uvs = vec![vec2(0., 0.), vec2(1., 0.)];

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    // .with_indices(Some(Indices::U32(indices)))
}

// fn generate_circle_mesh(sides: usize, radius: f32) -> Mesh {
//     let mut positions = Vec::with_capacity(sides);
//     let mut normals = Vec::with_capacity(sides);
//     let mut uvs = Vec::with_capacity(sides);

//     let step = std::f32::consts::TAU / sides as f32;
//     for i in 0..sides {
//         let theta = std::f32::consts::FRAC_PI_2 - i as f32 * step;
//         let (sin, cos) = theta.sin_cos();

//         let x = cos * radius;
//         let y = sin * radius;

//         positions.push([x, y, 0.0]);
//         normals.push([x / radius, y / radius, 0.]);
//         uvs.push([i as f32 / sides as f32, 0.]);
//     }

//     let mut indices = Vec::with_capacity((sides - 2) * 3);
//     for i in 1..(sides as u32 - 1) {
//         indices.extend_from_slice(&[0, i + 1, i]);
//     }

//     Mesh::new(PrimitiveTopology::TriangleList)
//         .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
//         .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
//         .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
//         .with_indices(Some(Indices::U32(indices)))
// }

impl Material for DebugMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/debug.wgsl".into()
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DebugMaterial {}
