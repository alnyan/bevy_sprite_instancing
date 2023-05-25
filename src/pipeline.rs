use std::mem::size_of;

use bevy::{
    prelude::{FromWorld, Resource, World},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BufferBindingType, RenderPipelineDescriptor, SamplerBindingType, ShaderStages,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, TextureSampleType,
            TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexStepMode,
        },
        renderer::RenderDevice,
    },
    sprite::{Mesh2dPipeline, Mesh2dPipelineKey},
};
use field_offset::offset_of;

use super::{shader::INSTANCED_ENTITY_SHADER_HANDLE, SpriteInstanceData};

#[derive(Resource)]
pub(super) struct InstancedSpritePipeline {
    pub spritesheet_uniform_layout: BindGroupLayout,
    pub mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for InstancedSpritePipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let descriptor = BindGroupLayoutDescriptor {
            label: Some("Instanced entity spritesheet bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };

        let spritesheet_uniform_layout = device.create_bind_group_layout(&descriptor);

        Self {
            spritesheet_uniform_layout,
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

impl SpecializedMeshPipeline for InstancedSpritePipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh2d_pipeline.specialize(key, layout)?;
        let shader = INSTANCED_ENTITY_SHADER_HANDLE.typed();

        descriptor.vertex.shader = shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = shader;

        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<SpriteInstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                // i_position
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: offset_of!(SpriteInstanceData => i_position).get_byte_offset() as u64,
                    shader_location: 2,
                },
                // i_scale
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: offset_of!(SpriteInstanceData => i_scale).get_byte_offset() as u64,
                    shader_location: 3,
                },
                // i_tex_index
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: offset_of!(SpriteInstanceData => i_tex_index).get_byte_offset() as u64,
                    shader_location: 4,
                },
            ],
        });

        descriptor.layout[1] = self.spritesheet_uniform_layout.clone();

        Ok(descriptor)
    }
}
