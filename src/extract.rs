use bevy_asset::Handle;
use bevy_ecs::prelude::*;
use bevy_math::Vec3Swizzles;
use bevy_render::{
    render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
    renderer::RenderDevice,
    texture::Image,
    Extract,
};
use bevy_transform::prelude::Transform;

use crate::SpriteInstanceData;

use super::{InstancedSprite, InstancedSpritesheet, SpriteInstancingGroup};

#[derive(Component)]
pub struct ExtractedSpriteInstancingBuffer {
    pub(super) device_buffer: Buffer,
    pub(super) length: usize,
}

#[derive(Component)]
pub struct ExtractedInstancedSpritesheet {
    pub(super) size_buffer: Buffer,
    pub(super) image: Handle<Image>,
}

impl ExtractedSpriteInstancingBuffer {
    pub fn upload(device: &RenderDevice, data: &[SpriteInstanceData]) -> Self {
        // TODO: find a way to write a buffer without allocating it each time
        //       (wgpu prevents MAP_WRITE buffers from being VERTEX buffers, which is weird for
        //       Vulkan)
        let device_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Entity instancing device buffer"),
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(data),
        });
        let length = data.len();

        Self {
            device_buffer,
            length,
        }
    }
}

pub(super) fn extract_instanced_spritesheets(
    mut commands: Commands,
    spritesheet_query: Extract<Query<(Entity, &InstancedSpritesheet)>>,
    device: Res<RenderDevice>,
) {
    for (id, spritesheet) in &spritesheet_query {
        let size = [spritesheet.width_tiles, spritesheet.height_tiles];

        let size_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Instanced spritesheet size uniform"),
            contents: bytemuck::cast_slice(&size),
            usage: BufferUsages::UNIFORM,
        });

        commands
            .get_or_spawn(id)
            .insert(ExtractedInstancedSpritesheet {
                size_buffer,
                image: spritesheet.image.clone(),
            });
    }
}

pub(super) fn extract_instanced_sprites(
    mut commands: Commands,
    entity_query: Extract<Query<(&Transform, &InstancedSprite)>>,
    instancing_groups: Extract<Query<(Entity, &SpriteInstancingGroup)>>,
    device: Res<RenderDevice>,
) {
    if entity_query.is_empty() {
        return;
    }

    for (id, group) in &instancing_groups {
        assert!(!group.entities.is_empty());

        let mut instancing_data = vec![];

        for entity in &group.entities {
            let (transform, instance) = entity_query.get(*entity).unwrap();

            instancing_data.push(SpriteInstanceData {
                i_position: transform.translation,
                i_scale: transform.scale.xy(),
                i_tex_index: instance.texture_index,
            });
        }

        commands
            .get_or_spawn(id)
            .insert(ExtractedSpriteInstancingBuffer::upload(
                &device,
                &instancing_data,
            ));
    }
}
