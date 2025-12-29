// Solar Voyage rendering shader
// Combines celestial body rendering, spacetime grid, and relativistic effects

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec4<f32>,
}

struct SimulationUniform {
    time: f32,
    ship_gamma: f32,     // Lorentz factor
    ship_speed_c: f32,   // Speed as fraction of c
    curvature_scale: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> sim: SimulationUniform;

// ============ Celestial Body Rendering ============

struct BodyInstance {
    @location(2) position: vec3<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
    @location(5) body_type: u32, // 0=Star, 1=Planet, 2=BlackHole, 3=Spaceship
}

struct BodyVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) body_type: u32,
}

@vertex
fn vs_body(
    @location(0) vertex_pos: vec2<f32>,
    instance: BodyInstance,
) -> BodyVertexOutput {
    var out: BodyVertexOutput;

    // Billboard facing camera
    let camera_right = vec3<f32>(camera.view[0][0], camera.view[1][0], camera.view[2][0]);
    let camera_up = vec3<f32>(camera.view[0][1], camera.view[1][1], camera.view[2][1]);

    let world_pos = instance.position
        + camera_right * vertex_pos.x * instance.radius
        + camera_up * vertex_pos.y * instance.radius;

    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = instance.color;
    out.local_pos = vertex_pos;
    out.world_pos = instance.position;
    out.body_type = instance.body_type;

    return out;
}

@fragment
fn fs_body(in: BodyVertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);

    if dist > 1.0 {
        discard;
    }

    // Special rendering for different body types
    if in.body_type == 0u { // Star
        // Glowing star effect
        let glow = exp(-dist * 2.0);
        let corona = exp(-dist * 0.5) * 0.3;
        return vec4<f32>(in.color.rgb * (glow + corona + 0.5), 1.0);
    } else if in.body_type == 2u { // Black Hole
        // Event horizon with accretion glow
        let event_horizon = 0.7;
        if dist < event_horizon {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Pure black center
        }
        // Accretion disk glow
        let ring = smoothstep(event_horizon, event_horizon + 0.1, dist) *
                   smoothstep(1.0, 0.8, dist);
        let glow_color = vec3<f32>(1.0, 0.6, 0.2) * ring * 2.0;
        return vec4<f32>(glow_color, ring);
    } else if in.body_type == 3u { // Spaceship
        // Simple ship with thruster
        let ship_shape = smoothstep(1.0, 0.3, dist);
        // Add thruster glow based on speed
        let thruster = smoothstep(0.0, 0.5, in.local_pos.y) *
                       smoothstep(1.0, 0.6, dist) * sim.ship_speed_c * 5.0;
        let thruster_color = vec3<f32>(0.3, 0.6, 1.0) * thruster;
        return vec4<f32>(in.color.rgb * ship_shape + thruster_color, max(ship_shape, thruster));
    } else { // Planet
        // Sphere shading
        let normal = vec3<f32>(in.local_pos, sqrt(max(0.0, 1.0 - dist * dist)));
        let light_dir = normalize(vec3<f32>(0.5, 0.5, 1.0));
        let diffuse = max(dot(normal, light_dir), 0.0);
        let ambient = 0.15;
        return vec4<f32>(in.color.rgb * (ambient + diffuse * 0.85), 1.0);
    }
}

// ============ Trail Rendering ============

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

// ============ Spacetime Grid ============

struct GridVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct GridOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_y: f32,
}

@vertex
fn vs_grid(in: GridVertex) -> GridOutput {
    var out: GridOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.world_y = in.position.y;
    return out;
}

@fragment
fn fs_grid(in: GridOutput) -> @location(0) vec4<f32> {
    // Depth-based intensity (deeper = brighter, showing curvature)
    let depth_intensity = clamp(-in.world_y * 0.5, 0.0, 1.0);
    var color = in.color;
    color.rgb = mix(color.rgb, vec3<f32>(1.0, 0.3, 0.1), depth_intensity * 0.5);
    return color;
}

// ============ Lensing Rays ============

@vertex
fn vs_lens(in: TrailVertex) -> TrailOutput {
    var out: TrailOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_lens(in: TrailOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// ============ Starfield Background ============

@vertex
fn vs_skybox(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
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
    let resolution = vec2<f32>(1280.0, 720.0);
    let uv = frag_coord.xy / resolution;

    // Reconstruct view direction
    let ndc = uv * 2.0 - 1.0;
    let dir = normalize(vec3<f32>(ndc.x * (resolution.x / resolution.y), ndc.y, 1.0));

    // Relativistic aberration: stars appear to bunch up in direction of travel
    // (simplified effect based on ship speed)
    var aberrated_dir = dir;
    if sim.ship_speed_c > 0.01 {
        let compression = 1.0 + sim.ship_speed_c * 0.5;
        aberrated_dir.z *= compression;
        aberrated_dir = normalize(aberrated_dir);
    }

    // Procedural star field
    var star_brightness = 0.0;
    let grid_scale = 80.0;
    let cell = floor(aberrated_dir * grid_scale);

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
                let angular_dist = acos(clamp(dot(aberrated_dir, star_dir), -1.0, 1.0));

                let star_size = 0.001 + hash(current_cell + vec3<f32>(0.5)) * 0.003;
                let brightness = hash(current_cell + vec3<f32>(0.7));

                if angular_dist < star_size && brightness > 0.8 {
                    star_brightness = max(star_brightness, (1.0 - angular_dist / star_size) * brightness);
                }
            }
        }
    }

    // Relativistic Doppler shift: blue shift in front, red shift behind
    var star_color = vec3<f32>(1.0, 0.98, 0.95);
    if sim.ship_speed_c > 0.1 {
        let forward = aberrated_dir.z;
        if forward > 0.0 {
            // Blue shift
            star_color = mix(star_color, vec3<f32>(0.7, 0.8, 1.0), forward * sim.ship_speed_c);
        } else {
            // Red shift
            star_color = mix(star_color, vec3<f32>(1.0, 0.7, 0.5), -forward * sim.ship_speed_c);
        }
    }

    // Background with subtle gradient
    let bg = vec3<f32>(0.005, 0.005, 0.015);
    let milky_way = smoothstep(-0.3, 0.3, aberrated_dir.y) * 0.02;

    return vec4<f32>(bg + milky_way + star_color * star_brightness * 1.5, 1.0);
}

// ============ HUD Elements ============

struct HudVertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct HudOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_hud(in: HudVertex) -> HudOutput {
    var out: HudOutput;
    // Convert from screen space to clip space
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_hud(in: HudOutput) -> @location(0) vec4<f32> {
    return in.color;
}
