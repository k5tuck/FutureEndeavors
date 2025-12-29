// Quantum simulation shader
// Supports point clouds, lines, and wavefunction visualization

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// ============================================
// Point Cloud Rendering (Orbitals, Particles)
// ============================================

struct PointInstance {
    @location(2) world_position: vec3<f32>,
    @location(3) size: f32,
    @location(4) color: vec4<f32>,
}

struct PointVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
}

@vertex
fn vs_point(
    @location(0) vertex_pos: vec2<f32>,
    instance: PointInstance,
) -> PointVertexOutput {
    var out: PointVertexOutput;

    // Billboard quad facing camera
    let world_pos = vec4<f32>(
        instance.world_position.xy + vertex_pos * instance.size,
        instance.world_position.z,
        1.0
    );

    out.clip_position = camera.view_proj * world_pos;
    out.color = instance.color;
    out.local_pos = vertex_pos;

    return out;
}

@fragment
fn fs_point(in: PointVertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);

    // Discard pixels outside circle
    if dist > 1.0 {
        discard;
    }

    // Soft glow effect for probability cloud
    let core_intensity = exp(-dist * dist * 2.0);
    let glow = 1.0 - smoothstep(0.0, 1.0, dist);

    let final_color = in.color.rgb * (core_intensity * 0.7 + 0.3);
    let final_alpha = in.color.a * glow;

    return vec4<f32>(final_color, final_alpha);
}

// ============================================
// Line Rendering (Edges, Connections)
// ============================================

struct LineVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct LineVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_line(in: LineVertexInput) -> LineVertexOutput {
    var out: LineVertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_line(in: LineVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// ============================================
// Wavefunction Rendering (1D/2D plots)
// ============================================

struct WaveVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) value: f32,
    @location(2) phase: f32,
}

struct WaveVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) value: f32,
    @location(1) phase: f32,
}

@vertex
fn vs_wave(in: WaveVertexInput) -> WaveVertexOutput {
    var out: WaveVertexOutput;

    // Convert 2D position to clip space
    let pos_3d = vec4<f32>(in.position, 0.0, 1.0);
    out.clip_position = camera.view_proj * pos_3d;
    out.value = in.value;
    out.phase = in.phase;

    return out;
}

@fragment
fn fs_wave(in: WaveVertexOutput) -> @location(0) vec4<f32> {
    // Color based on phase (rainbow)
    let hue = (in.phase + 3.14159) / 6.28318;
    let rgb = hsv_to_rgb(hue, 0.9, 1.0);

    // Intensity based on probability
    let intensity = abs(in.value);

    return vec4<f32>(rgb * intensity, 1.0);
}

// ============================================
// Quark/Hadron Rendering
// ============================================

struct QuarkVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) is_antiquark: f32,
}

@vertex
fn vs_quark(
    @location(0) vertex_pos: vec2<f32>,
    instance: PointInstance,
) -> QuarkVertexOutput {
    var out: QuarkVertexOutput;

    let world_pos = vec4<f32>(
        instance.world_position.xy + vertex_pos * instance.size,
        instance.world_position.z,
        1.0
    );

    out.clip_position = camera.view_proj * world_pos;
    out.color = instance.color;
    out.local_pos = vertex_pos;
    out.is_antiquark = 0.0; // Could be passed via instance data

    return out;
}

@fragment
fn fs_quark(in: QuarkVertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);

    if dist > 1.0 {
        discard;
    }

    // Solid core with color charge
    let core = 1.0 - smoothstep(0.0, 0.6, dist);
    let edge = 1.0 - smoothstep(0.6, 1.0, dist);

    // Gluon field visualization (swirling pattern)
    let angle = atan2(in.local_pos.y, in.local_pos.x);
    let field = sin(angle * 3.0 + dist * 5.0) * 0.2 + 0.8;

    let color = in.color.rgb * (core + edge * field * 0.3);
    return vec4<f32>(color, in.color.a * edge);
}

// ============================================
// Flux Tube Rendering (QCD strings)
// ============================================

@fragment
fn fs_flux_tube(in: LineVertexOutput) -> @location(0) vec4<f32> {
    // Gradient along the tube with pulsing effect
    let color = in.color.rgb;
    return vec4<f32>(color, in.color.a * 0.8);
}

// ============================================
// Probability Density Rendering
// ============================================

@fragment
fn fs_probability(in: PointVertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);

    if dist > 1.0 {
        discard;
    }

    // Probability cloud with quantum interference pattern
    let density = exp(-dist * dist * 3.0);
    let interference = sin(dist * 10.0) * 0.1 + 0.9;

    let color = in.color.rgb * density * interference;
    let alpha = in.color.a * (1.0 - dist);

    return vec4<f32>(color, alpha);
}

// ============================================
// Utility Functions
// ============================================

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let h6 = h * 6.0;
    let i = floor(h6);
    let f = h6 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let i_mod = i32(i) % 6;

    if i_mod == 0 {
        return vec3<f32>(v, t, p);
    } else if i_mod == 1 {
        return vec3<f32>(q, v, p);
    } else if i_mod == 2 {
        return vec3<f32>(p, v, t);
    } else if i_mod == 3 {
        return vec3<f32>(p, q, v);
    } else if i_mod == 4 {
        return vec3<f32>(t, p, v);
    } else {
        return vec3<f32>(v, p, q);
    }
}

// Phase to color for wavefunction visualization
fn phase_to_color(phase: f32) -> vec3<f32> {
    // Map phase [-π, π] to hue [0, 1]
    let hue = (phase + 3.14159) / 6.28318;
    return hsv_to_rgb(hue, 0.9, 1.0);
}

// Complex magnitude visualization
fn complex_to_color(re: f32, im: f32) -> vec4<f32> {
    let magnitude = sqrt(re * re + im * im);
    let phase = atan2(im, re);
    let rgb = phase_to_color(phase);
    return vec4<f32>(rgb * magnitude, magnitude);
}
