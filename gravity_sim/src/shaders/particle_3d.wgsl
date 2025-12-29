// 3D Particle rendering shader with depth and lighting

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Particle instance data
struct ParticleInstance {
    @location(2) world_position: vec3<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
}

@vertex
fn vs_particle_3d(
    @location(0) vertex_pos: vec2<f32>,
    instance: ParticleInstance,
) -> VertexOutput {
    var out: VertexOutput;

    // Billboard: face camera
    let camera_right = vec3<f32>(camera.view[0][0], camera.view[1][0], camera.view[2][0]);
    let camera_up = vec3<f32>(camera.view[0][1], camera.view[1][1], camera.view[2][1]);

    let world_pos = instance.world_position
        + camera_right * vertex_pos.x * instance.radius
        + camera_up * vertex_pos.y * instance.radius;

    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = instance.color;
    out.local_pos = vertex_pos;
    out.world_pos = instance.world_position;

    return out;
}

@fragment
fn fs_particle_3d(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);

    if dist > 1.0 {
        discard;
    }

    // Sphere shading
    let normal = vec3<f32>(in.local_pos, sqrt(max(0.0, 1.0 - dist * dist)));

    // Lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let diffuse = max(dot(normal, light_dir), 0.0);
    let ambient = 0.2;

    // View direction for specular
    let view_dir = normalize(camera.position.xyz - in.world_pos);
    let reflect_dir = reflect(-light_dir, normal);
    let specular = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0) * 0.4;

    let lit_color = in.color.rgb * (ambient + diffuse * 0.8) + vec3<f32>(specular);

    // Glow effect for stars/massive objects
    let glow = exp(-dist * 2.0) * in.color.a * 0.3;

    return vec4<f32>(lit_color + in.color.rgb * glow, 1.0);
}

// Trail rendering for orbits
struct TrailVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct TrailOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_trail(in: TrailVertex) -> TrailOutput {
    var out: TrailOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_trail(in: TrailOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// Grid for reference
struct GridOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_grid(@builtin(vertex_index) vertex_index: u32) -> GridOutput {
    let grid_size = 21u;
    let grid_extent = 50.0;

    let line_index = vertex_index / 2u;
    let is_start = vertex_index % 2u == 0u;

    var pos: vec3<f32>;
    var color: vec4<f32>;

    if line_index < grid_size {
        // X-axis lines
        let t = f32(line_index) / f32(grid_size - 1u) * 2.0 - 1.0;
        let x = select(grid_extent, -grid_extent, is_start);
        pos = vec3<f32>(x, 0.0, t * grid_extent);
        color = select(vec4<f32>(0.3, 0.3, 0.3, 0.5), vec4<f32>(0.5, 0.2, 0.2, 0.8), line_index == grid_size / 2u);
    } else {
        // Z-axis lines
        let idx = line_index - grid_size;
        let t = f32(idx) / f32(grid_size - 1u) * 2.0 - 1.0;
        let z = select(grid_extent, -grid_extent, is_start);
        pos = vec3<f32>(t * grid_extent, 0.0, z);
        color = select(vec4<f32>(0.3, 0.3, 0.3, 0.5), vec4<f32>(0.2, 0.2, 0.5, 0.8), idx == grid_size / 2u);
    }

    var out: GridOutput;
    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_grid(in: GridOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// Skybox / star field
@vertex
fn vs_skybox(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Full-screen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(positions[vertex_index], 0.9999, 1.0);
}

fn hash(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

@fragment
fn fs_skybox(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // Simple procedural stars
    let uv = frag_coord.xy / vec2<f32>(1280.0, 720.0);
    let dir = normalize(vec3<f32>(uv * 2.0 - 1.0, 1.0));

    var star_brightness = 0.0;
    let grid_scale = 100.0;
    let cell = floor(dir * grid_scale);

    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            for (var k = -1; k <= 1; k++) {
                let offset = vec3<f32>(f32(i), f32(j), f32(k));
                let current_cell = cell + offset;

                let star_pos = (current_cell + vec3<f32>(
                    hash(current_cell),
                    hash(current_cell + vec3<f32>(1.0, 0.0, 0.0)),
                    hash(current_cell + vec3<f32>(0.0, 1.0, 0.0))
                )) / grid_scale;

                let star_dir = normalize(star_pos);
                let angular_dist = acos(clamp(dot(dir, star_dir), -1.0, 1.0));

                let star_size = 0.001 + hash(current_cell + vec3<f32>(0.5)) * 0.002;
                let brightness = hash(current_cell + vec3<f32>(0.7));

                if angular_dist < star_size && brightness > 0.85 {
                    star_brightness = max(star_brightness, (1.0 - angular_dist / star_size) * brightness);
                }
            }
        }
    }

    let bg_color = vec3<f32>(0.01, 0.01, 0.02);
    let star_color = vec3<f32>(1.0, 0.95, 0.9) * star_brightness;

    return vec4<f32>(bg_color + star_color, 1.0);
}
