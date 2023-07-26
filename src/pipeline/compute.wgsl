
struct Globals {
    time: f32,
}

fn mandelbrot(p: vec2<f32>, max_iterations: u32) -> f32 {
    var i: u32 = 0u;
    var a: f32 = 0.0;
    var b: f32 = 0.0;
    while (a*a + b*b < 4.0 && i < max_iterations) {
        let a_ = a*a - b*b + p.x;
        let b_ = 2.0*a*b + p.y;
        a = a_;
        b = b_;
        i = i + 1u;
    }
    return f32(i) / f32(max_iterations);
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
    let aspect_ratio = f32(dimensions.x) / f32(dimensions.y);
    let uv = vec2<f32>(
        f32(g_invocation_id.x) / f32(dimensions.x),
        f32(g_invocation_id.y) / f32(dimensions.y)
    );

    // TODO: Move these to uniforms
    let viewport_scale = 4.0;
    let viewport_center = vec2<f32>(-0.34853774148008254, -0.6065922085831237);

    let viewport_size = vec2<f32>(1.0, 1.0 / aspect_ratio) * viewport_scale;
    let euclidean_point = (uv - 0.5) * viewport_size + viewport_center;

    let color = mandelbrot(euclidean_point, 2048u);

    textureStore(tex, g_invocation_id.xy, vec4<f32>(color * ((sin(globals.time) + 1.0) / 2.0), 0.0, color, 1.0));
}
