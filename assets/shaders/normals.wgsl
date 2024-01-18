#import bevy_pbr::forward_io::VertexOutput

// struct NormalsMaterial {
// };

// @group(1) @binding(0) var<uniform> material: NormalsMaterial;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return vec4(mesh.world_normal, 1.);
}