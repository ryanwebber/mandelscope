
struct Globals {
    time: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var tex: texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(1, 1, 1)
fn main(
    @builtin(global_invocation_id) g_invocation_id: vec3<u32>
) {
    let dimensions = textureDimensions(tex);
    let pixel_index = g_invocation_id.y * dimensions.x + g_invocation_id.x;
    let pixel_coords = vec2<u32>(g_invocation_id.x, g_invocation_id.y);

    let uv = vec2<f32>(
        f32(pixel_coords.x) / f32(dimensions.x),
        f32(pixel_coords.y) / f32(dimensions.y)
    );

    let timeshift = (vec2<f32>(sin(globals.time), cos(globals.time)) + 1.0) * 0.5;

    textureStore(tex, g_invocation_id.xy, vec4<f32>(uv * timeshift, 1.0, 1.0));
}
