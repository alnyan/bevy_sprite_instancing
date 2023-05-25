use bevy_asset::HandleUntyped;
use bevy_reflect::TypeUuid;
use bevy_render::render_resource::Shader;

pub const INSTANCED_ENTITY_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12344143414121);

pub(super) const INSTANCED_ENTITY_SHADER: &str = r#"
#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings

#import bevy_sprite::mesh2d_functions

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;
@group(1) @binding(2)
var<uniform> spritesheet_tile_size: vec2<u32>;

struct Vertex {
    // Per-vertex
    @location(0) v_position: vec2<f32>,
    @location(1) v_tex_coords: vec2<f32>,

    // Per-instance
    @location(2) i_position: vec3<f32>,
    @location(3) i_scale: vec2<f32>,
    @location(4) i_tex_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) m_tex_coords: vec2<f32>,
    @location(1) m_tex_index: u32
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let position_2d = vertex.v_position * vertex.i_scale + vertex.i_position.xy;
    let position_ws = vec4(position_2d, vertex.i_position.z, 1.0);

    out.clip_position = mesh2d_position_world_to_clip(position_ws);
    out.m_tex_coords = vertex.v_tex_coords;
    out.m_tex_index = vertex.i_tex_index;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let tile_size_f32 = vec2(f32(spritesheet_tile_size.x), f32(spritesheet_tile_size.y));

    let t_u = f32(in.m_tex_index % spritesheet_tile_size.x) / tile_size_f32.x;
    let t_v = f32(in.m_tex_index / spritesheet_tile_size.x) / tile_size_f32.y;
    let tex_coords = in.m_tex_coords / tile_size_f32 + vec2(t_u, t_v);

    let tex_color = textureSample(texture, texture_sampler, tex_coords);
    return tex_color;
}
"#;
