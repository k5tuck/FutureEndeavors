// Black hole visualization shader
// Implements gravitational lensing via ray marching

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
}

struct BlackHoleUniform {
    position: vec4<f32>,
    mass: f32,
    schwarzschild_radius: f32,
    time: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> black_hole: BlackHoleUniform;

// Vertex shader for full-screen quad (ray marching)
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate full-screen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = positions[vertex_index] * 0.5 + 0.5;
    return out;
}

// Constants
const PI: f32 = 3.14159265359;
const G: f32 = 1.0;
const C: f32 = 1.0;

// Star field hash function
fn hash(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Star field
fn star_field(dir: vec3<f32>) -> vec3<f32> {
    let grid_scale = 50.0;
    let cell = floor(dir * grid_scale);

    var brightness = 0.0;
    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            for (var k = -1; k <= 1; k++) {
                let offset = vec3<f32>(f32(i), f32(j), f32(k));
                let current_cell = cell + offset;

                // Random star position within cell
                let star_pos = (current_cell + vec3<f32>(
                    hash(current_cell),
                    hash(current_cell + vec3<f32>(1.0, 0.0, 0.0)),
                    hash(current_cell + vec3<f32>(0.0, 1.0, 0.0))
                )) / grid_scale;

                let star_dir = normalize(star_pos);
                let angular_dist = acos(clamp(dot(dir, star_dir), -1.0, 1.0));

                // Star appearance
                let star_size = 0.002 + hash(current_cell + vec3<f32>(0.5)) * 0.003;
                let star_brightness = hash(current_cell + vec3<f32>(0.7));

                if angular_dist < star_size && star_brightness > 0.7 {
                    brightness = max(brightness, (1.0 - angular_dist / star_size) * star_brightness);
                }
            }
        }
    }

    // Star color variation
    let temp = 0.5 + hash(dir * 1000.0) * 0.5;
    let color = mix(
        vec3<f32>(1.0, 0.8, 0.6), // warm
        vec3<f32>(0.8, 0.9, 1.0), // cool
        temp
    );

    return color * brightness * 2.0;
}

// Accretion disk color
fn accretion_disk_color(pos: vec3<f32>, bh_pos: vec3<f32>, rs: f32) -> vec4<f32> {
    let inner_radius = rs * 3.0;
    let outer_radius = rs * 15.0;

    // Distance in xz plane from black hole
    let r = length(pos.xz - bh_pos.xz);
    let y_dist = abs(pos.y - bh_pos.y);

    // Disk thickness increases with radius
    let thickness = 0.1 + (r - inner_radius) * 0.02;

    if r < inner_radius || r > outer_radius || y_dist > thickness {
        return vec4<f32>(0.0);
    }

    // Temperature based on radius (hotter near center)
    let t = (r - inner_radius) / (outer_radius - inner_radius);
    let temp = mix(1.0, 0.3, t);

    // Color from temperature
    let color = mix(
        vec3<f32>(1.0, 0.9, 0.7), // hot yellow-white
        vec3<f32>(1.0, 0.3, 0.1), // cool red-orange
        t
    );

    // Intensity falloff with height
    let height_falloff = 1.0 - (y_dist / thickness);

    // Orbital velocity creates doppler shift (simplified)
    let angle = atan2(pos.z - bh_pos.z, pos.x - bh_pos.x);
    let orbital_phase = sin(angle * 4.0 + black_hole.time * 2.0) * 0.2 + 0.8;

    return vec4<f32>(color * temp * height_falloff * orbital_phase, 1.0);
}

// Ray march through curved spacetime
fn trace_ray(origin: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let bh_pos = black_hole.position.xyz;
    let rs = black_hole.schwarzschild_radius;
    let mass = black_hole.mass;

    var pos = origin;
    var vel = normalize(dir);
    let step_size = 0.1;
    let max_steps = 500u;

    var accumulated_color = vec3<f32>(0.0);
    var opacity = 0.0;

    for (var i = 0u; i < max_steps; i++) {
        let r_vec = pos - bh_pos;
        let r = length(r_vec);

        // Captured by black hole
        if r < rs * 1.1 {
            return vec3<f32>(0.0, 0.0, 0.0); // Pure black
        }

        // Check accretion disk
        let disk_color = accretion_disk_color(pos, bh_pos, rs);
        if disk_color.a > 0.0 && opacity < 0.95 {
            let blend = disk_color.a * (1.0 - opacity);
            accumulated_color += disk_color.rgb * blend;
            opacity += blend;
        }

        // Escaped to infinity
        if r > 50.0 {
            if opacity < 0.95 {
                let stars = star_field(vel);
                accumulated_color += stars * (1.0 - opacity);
            }
            return accumulated_color;
        }

        // Gravitational deflection
        let r_hat = r_vec / r;
        let photon_sphere = 1.5 * rs;
        var enhancement = 1.0;
        if r < photon_sphere * 2.0 {
            enhancement = 1.0 + 3.0 * pow(photon_sphere / r, 2.0);
        }

        let accel = -r_hat * (G * mass / (r * r)) * enhancement * 3.0;

        // Velocity Verlet integration
        let new_vel = normalize(vel + accel * step_size);
        pos += (vel + new_vel) * 0.5 * step_size;
        vel = new_vel;
    }

    return accumulated_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to ray direction
    let aspect = 16.0 / 9.0;
    let fov = 1.0;

    let ray_dir = normalize(vec3<f32>(
        (in.uv.x - 0.5) * 2.0 * aspect * fov,
        (in.uv.y - 0.5) * 2.0 * fov,
        -1.0
    ));

    // Transform ray by camera
    let cam_pos = camera.position.xyz;

    // Simple ray tracing
    let color = trace_ray(cam_pos, ray_dir);

    return vec4<f32>(color, 1.0);
}

// Vertex shader for light ray visualization (2D mode)
struct LineVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct LineVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_line(in: LineVertexInput) -> LineVertexOutput {
    var out: LineVertexOutput;
    out.position = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_line(in: LineVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// Event horizon circle shader
@vertex
fn vs_circle(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> LineVertexOutput {
    let segments = 64u;
    let angle = f32(vertex_index) / f32(segments) * 2.0 * PI;

    let radius = select(
        black_hole.schwarzschild_radius,
        black_hole.schwarzschild_radius * 1.5, // photon sphere
        instance_index == 1u
    );

    let pos = black_hole.position.xy + vec2<f32>(cos(angle), sin(angle)) * radius;

    var out: LineVertexOutput;
    out.position = camera.view_proj * vec4<f32>(pos, 0.0, 1.0);
    out.color = select(
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
        vec4<f32>(1.0, 0.5, 0.0, 0.5),
        instance_index == 1u
    );
    return out;
}

@fragment
fn fs_circle(in: LineVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
