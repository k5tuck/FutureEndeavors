// Atom rendering shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Atom instance data
struct AtomInstance {
    @location(2) position: vec3<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
}

@vertex
fn vs_atom(
    @location(0) vertex_pos: vec2<f32>,
    instance: AtomInstance,
) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = vec4<f32>(
        instance.position.xy + vertex_pos * instance.radius,
        instance.position.z,
        1.0
    );

    out.clip_position = camera.view_proj * world_pos;
    out.color = instance.color;
    out.local_pos = vertex_pos;

    return out;
}

@fragment
fn fs_atom(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);

    if dist > 1.0 {
        discard;
    }

    // 3D sphere shading using the local position as a normal
    let normal = vec3<f32>(in.local_pos, sqrt(max(0.0, 1.0 - dist * dist)));

    // Simple lighting
    let light_dir = normalize(vec3<f32>(0.5, 0.5, 1.0));
    let diffuse = max(dot(normal, light_dir), 0.0);
    let ambient = 0.3;

    let lit_color = in.color.rgb * (ambient + diffuse * 0.7);

    // Specular highlight
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let reflect_dir = reflect(-light_dir, normal);
    let specular = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0) * 0.5;

    let final_color = lit_color + vec3<f32>(specular);

    // Rim lighting for edge definition
    let rim = 1.0 - max(dot(normal, view_dir), 0.0);
    let rim_color = in.color.rgb * rim * 0.3;

    return vec4<f32>(final_color + rim_color, in.color.a);
}

// Bond rendering
struct BondVertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct BondVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_bond(in: BondVertex) -> BondVertexOutput {
    var out: BondVertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_bond(in: BondVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// Element symbol rendering (simplified)
struct SymbolInstance {
    @location(2) position: vec2<f32>,
    @location(3) char_index: u32,
}

// Background grid
@vertex
fn vs_grid(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Generate grid lines
    let grid_size = 10u;
    let line_index = vertex_index / 2u;
    let is_horizontal = line_index < grid_size;
    let line_num = select(line_index - grid_size, line_index, is_horizontal);

    let t = f32(line_num) / f32(grid_size - 1u) * 2.0 - 1.0;
    let extent = 15.0;

    var pos: vec2<f32>;
    if is_horizontal {
        let x = select(-extent, extent, vertex_index % 2u == 1u);
        pos = vec2<f32>(x, t * extent);
    } else {
        let y = select(-extent, extent, vertex_index % 2u == 1u);
        pos = vec2<f32>(t * extent, y);
    }

    return camera.view_proj * vec4<f32>(pos, -0.1, 1.0);
}

@fragment
fn fs_grid() -> @location(0) vec4<f32> {
    return vec4<f32>(0.1, 0.1, 0.15, 1.0);
}
