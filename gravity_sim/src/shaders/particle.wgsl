// Particle rendering shader for gravity simulation

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) center: vec2<f32>,
    @location(2) point_coord: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.center = out.clip_position.xy / out.clip_position.w;
    out.point_coord = vec2<f32>(0.0, 0.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple circle rendering with soft edges
    let dist = length(in.point_coord);
    if dist > 1.0 {
        discard;
    }

    let alpha = 1.0 - smoothstep(0.7, 1.0, dist);
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

// Instanced particle rendering
struct ParticleInstance {
    @location(2) world_position: vec3<f32>,
    @location(3) radius: f32,
    @location(4) inst_color: vec4<f32>,
}

struct InstanceVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
}

@vertex
fn vs_instanced(
    @location(0) vertex_pos: vec2<f32>,
    instance: ParticleInstance,
) -> InstanceVertexOutput {
    var out: InstanceVertexOutput;

    // Billboard: always face camera
    let world_pos = vec4<f32>(
        instance.world_position.xy + vertex_pos * instance.radius,
        instance.world_position.z,
        1.0
    );

    out.clip_position = camera.view_proj * world_pos;
    out.color = instance.inst_color;
    out.local_pos = vertex_pos;

    return out;
}

@fragment
fn fs_instanced(in: InstanceVertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);
    if dist > 1.0 {
        discard;
    }

    // Smooth circular edge with glow effect
    let core = 1.0 - smoothstep(0.0, 0.5, dist);
    let edge = 1.0 - smoothstep(0.5, 1.0, dist);

    let glow = in.color.rgb * (core * 0.5 + 0.5);
    return vec4<f32>(glow, in.color.a * edge);
}
