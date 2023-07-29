
// ======================== Structs =======================

struct Globals {
    time: f32,
    scale: f32,
    radius: f32,
    center: vec2<f32>,
    orbit_offset: vec2<f32>,
    coefficients: array<vec4<f32>, 4>,
}

struct OrbitBuffer {
    iterations: u32,
    orbits: array<vec2<f32>>,
}

// ================== Mandelbrot function =================

fn mandelbrot(d0: vec2<f32>) -> f32 {
    let a = globals.coefficients[0u].xy;
    let b = globals.coefficients[1u].xy;
    let c = globals.coefficients[2u].xy;
    let d = globals.coefficients[3u].xy;

    var dn = cxmul(a, d0) + cxmul(b, cxpow(d0, 2.0)) + cxmul(c, cxpow(d0, 3.0)) + cxmul(d, cxpow(d0, 4.0));
    var xn = vec2<f32>(0.0, 0.0);

    var i = 0u;
    for (; i < orbit_buffer.iterations; i += 1u) {
        xn = orbit_buffer.orbits[i];
        dn = cxmul(2.0 * xn + dn, dn) + d0;
        if (length(dn) > globals.radius) {
            break;
        }
    }

    return f32(i) / f32(orbit_buffer.iterations);
}

// ================ Complex Math functions ================

fn cxmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
}

fn cxpow(a: vec2<f32>, b: f32) -> vec2<f32> {
    let r = length(a);
    let theta = atan2(a.y, a.x);
    let rprime = pow(r, b);
    let thetaprime = theta * b;
    return vec2<f32>(
        rprime * cos(thetaprime),
        rprime * sin(thetaprime)
    );
}

// ========================= Main =========================

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var<storage, read> orbit_buffer: OrbitBuffer;

@group(0) @binding(2)
var tex: texture_storage_2d<rgba32float, read_write>;

@compute
@workgroup_size(1, 1, 1)
fn main(
    @builtin(global_invocation_id) g_invocation_id: vec3<u32>
) {
    let dimensions = textureDimensions(tex);
    let aspect_ratio = vec2<f32>(f32(dimensions.x) / f32(dimensions.y), 1.0);
    
    let uv = vec2<f32>(
        f32(g_invocation_id.x) / f32(dimensions.x),
        f32(g_invocation_id.y) / f32(dimensions.y)
    );

    let x = globals.scale * aspect_ratio * (uv - 0.5) - globals.orbit_offset;
    let color = vec3<f32>(mandelbrot(x));

    textureStore(tex, g_invocation_id.xy, vec4<f32>(color, 1.0));
}
