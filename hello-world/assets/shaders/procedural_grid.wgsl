// TODO: Figure out how many of these imports we actually need
#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::mesh_functions
#import bevy_pbr::prepass_utils

#import bevy_pbr::mesh_vertex_output MeshVertexOutput

fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    if (a == b) { return 1.0; }
    return (v - a) / (b - a);
}

fn grid_lines(pos: vec4<f32>, depth: f32, scale: f32, depth_min: f32, depth_max: f32) -> f32 {
    let ref_coords = abs(pos) * scale;

    // TODO: `modf` no longer takes a pointer as its second argument, but that
    // change is not yet reflected in the Naga version used at time of writing.
    // When Bevy updates its Naga dependency, this will need to be updated:
    // ```
    // let fracs = modf(ref_coords).fract;
    // ```
    // Reference: https://www.w3.org/TR/2023/WD-WGSL-20230927/#modf-builtin
    var wholes = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    modf(ref_coords, &wholes);
    let fracs = ref_coords - wholes;

    let thickness = (0.0333 * scale) / (depth * 100.0);

    var x = smoothstep(0.0, thickness, fracs.x);
    var z = smoothstep(0.0, thickness, fracs.z);
    let value = 1.0 - saturate(x * z);

    let mask = inv_lerp(depth_min, depth_max, depth);

    return saturate(value * mask);
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let depth = in.position.z;
    let pos = in.world_position;

    let dense = grid_lines(pos, depth,  1.0, 0.0001, 0.01);
    let sparse = grid_lines(pos, depth, 0.1, 0.00005, 0.005);

    return vec4<f32>(1.0, 1.0, 1.0, 1.0) * 0.5 * saturate(dense + sparse);
}
