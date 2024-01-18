use bevy::{
    math::vec3,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        mesh::Indices,
        render_resource::PrimitiveTopology,
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_flycam::PlayerPlugin;
use bevy_pcg_splines::{BevyPcgSplinesPlugin, PcgSpline};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
            }),
            BevyPcgSplinesPlugin,
            WireframePlugin,
            PlayerPlugin,
        ))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE,
        })
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

fn setup_curve(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let control_points = [[
        vec3(2.304, 4.156_404_5, 0.),
        vec3(3.23, 1.653_999_9, 0.),
        vec3(6.912, 3.210_370_3, 0.),
        vec3(9.216, 0.606_522_2, 0.),
    ]];

    // Create a new cubic Bézier curve from the control points
    let spline = CubicBezier::new(control_points);

    // Generate the curve which internally chains the segments
    let curve = spline.to_curve();

    // TODO: experiment with irregular shapes such as a donut
    commands.spawn(PcgSpline {
        mesh_2d: bevy::sprite::Mesh2dHandle(meshes.add(generate_circle_mesh(64, 0.5))),
        curve,
    });
}

fn generate_circle_mesh(sides: usize, radius: f32) -> Mesh {
    let mut positions = Vec::with_capacity(sides);
    let mut normals = Vec::with_capacity(sides);
    let mut uvs = Vec::with_capacity(sides);

    let step = std::f32::consts::TAU / sides as f32;
    for i in 0..sides {
        let theta = std::f32::consts::FRAC_PI_2 - i as f32 * step;
        let (sin, cos) = theta.sin_cos();

        let x = cos * radius;
        let y = sin * radius;

        positions.push([x, y, 0.0]);
        normals.push([x / radius, y / radius, 0.]);
        uvs.push([0.5 * (cos + 1.0), 1.0 - 0.5 * (sin + 1.0)]);
    }

    let mut indices = Vec::with_capacity((sides - 2) * 3);
    for i in 1..(sides as u32 - 1) {
        indices.extend_from_slice(&[0, i + 1, i]);
    }

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_indices(Some(Indices::U32(indices)))
}
