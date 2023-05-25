use std::collections::HashSet;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        mesh::{GpuBufferInfo, GpuMesh, MeshVertexAttribute},
        render_phase::AddRenderCommand,
        render_resource::{
            BindGroup, BufferInitDescriptor, BufferUsages, PrimitiveTopology,
            SpecializedMeshPipelines, VertexFormat,
        },
        renderer::RenderDevice,
        RenderApp, RenderSet,
    },
};
use bytemuck::{Pod, Zeroable};

use self::{
    draw::DrawSpritesInstancedCommands,
    extract::{extract_instanced_sprites, extract_instanced_spritesheets},
    pipeline::InstancedSpritePipeline,
    prepare::{prepare_instanced_spritesheets, queue_instanced_sprites},
    shader::{INSTANCED_ENTITY_SHADER, INSTANCED_ENTITY_SHADER_HANDLE},
};

mod draw;
mod extract;
mod pipeline;
mod prepare;
mod shader;

pub struct InstancedSpriteRenderPlugin;

#[derive(Component)]
pub struct InstancedSprite {
    pub texture_index: u32,
    pub group_id: Entity,
}

#[derive(Component)]
pub struct InstancedSpritesheet {
    // Width of the spritesheet in tiles
    pub width_tiles: u32,
    pub height_tiles: u32,
    pub image: Handle<Image>,
}

#[derive(Component)]
pub struct InstancedSpritesheetBindGroup {
    bind_group: BindGroup,
}

#[derive(Resource)]
pub struct InstancedSpriteMesh {
    quad: GpuMesh,
}

#[derive(Component)]
pub struct SpriteInstancingGroup {
    pub entities: HashSet<Entity>,
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct SpriteInstanceData {
    i_position: Vec3,
    i_scale: Vec2,
    i_tex_index: u32,
}

impl Plugin for InstancedSpriteRenderPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.set_untracked(
            INSTANCED_ENTITY_SHADER_HANDLE,
            Shader::from_wgsl(INSTANCED_ENTITY_SHADER),
        );

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_command::<Transparent2d, DrawSpritesInstancedCommands>()
            .init_resource::<InstancedSpritePipeline>()
            .init_resource::<SpecializedMeshPipelines<InstancedSpritePipeline>>()
            .add_system(
                setup_entity_instancing_mesh
                    .in_schedule(ExtractSchedule)
                    .run_if(not(resource_exists::<InstancedSpriteMesh>())),
            )
            .add_system(extract_instanced_spritesheets.in_schedule(ExtractSchedule))
            .add_system(extract_instanced_sprites.in_schedule(ExtractSchedule))
            .add_system(prepare_instanced_spritesheets.in_set(RenderSet::Prepare))
            .add_system(queue_instanced_sprites.in_set(RenderSet::Queue));
    }

    fn name(&self) -> &'static str {
        "Entity instanced render plugin"
    }
}

fn setup_entity_instancing_mesh(mut commands: Commands, device: Res<RenderDevice>) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let positions = vec![
        [0.5, -0.5, 0.0],  // bottom right
        [0.5, 0.5, 0.0],   // top right
        [-0.5, 0.5, 0.0],  // top left
        [-0.5, 0.5, 0.0],  // top left
        [-0.5, -0.5, 0.0], // bottom left
        [0.5, -0.5, 0.0],  // bottom right
    ];
    let uvs = vec![
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
    ];

    mesh.insert_attribute(
        MeshVertexAttribute::new("v_position", 0, VertexFormat::Float32x3),
        positions,
    );
    mesh.insert_attribute(
        MeshVertexAttribute::new("v_tex_coords", 1, VertexFormat::Float32x2),
        uvs,
    );

    let vertex_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Entity instancing quad mesh"),
        contents: &mesh.get_vertex_buffer_data(),
        usage: BufferUsages::VERTEX,
    });
    let buffer_info = GpuBufferInfo::NonIndexed {
        vertex_count: mesh.count_vertices() as u32,
    };
    let layout = mesh.get_mesh_vertex_buffer_layout();
    let primitive_topology = mesh.primitive_topology();

    let quad = GpuMesh {
        vertex_buffer,
        buffer_info,
        layout,
        primitive_topology,
    };

    commands.insert_resource(InstancedSpriteMesh { quad });
}
