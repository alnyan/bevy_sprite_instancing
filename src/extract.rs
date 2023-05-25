use std::mem::size_of;

use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{
        render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
        Extract,
    },
    utils::HashMap,
};

use super::{InstancedSprite, InstancedSpritesheet, SpriteInstanceData, SpriteInstancingGroup};

#[derive(Resource, Default)]
pub struct ExtractedComponentCache {
    instancing_buffers: HashMap<Entity, ExtractedSpriteInstancingBuffer>,
    spritesheets: HashMap<Entity, ExtractedInstancedSpritesheet>,
}

#[derive(Component, Clone)]
pub struct ExtractedSpriteInstancingBuffer {
    pub(super) device_buffer: Buffer,
    pub(super) length: usize,
    pub(super) capacity: usize,
}

#[derive(Component, Clone)]
pub struct ExtractedInstancedSpritesheet {
    pub(super) size_buffer: Buffer,
    pub(super) image: Handle<Image>,
}

impl ExtractedSpriteInstancingBuffer {
    const INITIAL_CAPACITY: usize = 1024;
    const CAPACITY_INCREMENT: usize = 512;

    pub fn new(device: &RenderDevice) -> Self {
        let device_buffer = Self::create_buffer(device, Self::INITIAL_CAPACITY);
        Self {
            device_buffer,
            length: 0,
            capacity: Self::INITIAL_CAPACITY,
        }
    }

    pub fn upload(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        data: &[SpriteInstanceData],
    ) {
        if data.len() > self.capacity {
            self.resize(device, data.len());
        }
        queue.write_buffer(&self.device_buffer, 0, bytemuck::cast_slice(data));
        self.length = data.len();
    }

    fn create_buffer(device: &RenderDevice, capacity: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("Entity instancing device buffer"),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            size: Self::buffer_size(capacity),
            mapped_at_creation: false,
        })
    }

    fn resize(&mut self, device: &RenderDevice, new_capacity: usize) {
        info!(
            "Grow entity instancing buffer, old capacity: {}, expected capacity: {}",
            self.capacity, new_capacity
        );
        while self.capacity < new_capacity {
            self.capacity += Self::CAPACITY_INCREMENT;
        }
        info!("Result capacity: {}", self.capacity);

        self.device_buffer = Self::create_buffer(device, self.capacity);
    }

    const fn buffer_size(n_elements: usize) -> u64 {
        (n_elements * size_of::<SpriteInstanceData>()) as u64
    }
}

impl ExtractedInstancedSpritesheet {
    pub fn new(device: &RenderDevice, spritesheet: &InstancedSpritesheet) -> Self {
        let size = [spritesheet.width_tiles, spritesheet.height_tiles];
        let size_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Instanced spritesheet size uniform"),
            contents: bytemuck::cast_slice(&size),
            usage: BufferUsages::UNIFORM,
        });

        Self {
            size_buffer,
            image: spritesheet.image.clone(),
        }
    }
}

impl ExtractedComponentCache {
    fn update_instancing_buffer(
        &mut self,
        entity: Entity,
        device: &RenderDevice,
        queue: &RenderQueue,
        data: &[SpriteInstanceData],
    ) -> ExtractedSpriteInstancingBuffer {
        let buffer = self.instancing_buffers.entry(entity).or_insert_with(|| {
            info!("Creating a new instance buffer {:?}", entity);
            ExtractedSpriteInstancingBuffer::new(device)
        });

        buffer.upload(device, queue, data);
        buffer.clone()
    }

    fn update_instanced_spritesheet(
        &mut self,
        entity: Entity,
        device: &RenderDevice,
        spritesheet: &InstancedSpritesheet,
    ) -> ExtractedInstancedSpritesheet {
        let spritesheet = self.spritesheets.entry(entity).or_insert_with(|| {
            info!("Creating a new spritesheet buffer {:?}", entity);
            ExtractedInstancedSpritesheet::new(device, spritesheet)
        });

        spritesheet.clone()
        // TODO handle spritesheet changes?
    }
}

pub(super) fn extract_instancing_groups(
    mut commands: Commands,
    entity_query: Extract<Query<(&Transform, &InstancedSprite)>>,
    instancing_groups: Extract<Query<(Entity, &InstancedSpritesheet, &SpriteInstancingGroup)>>,
    queue: Res<RenderQueue>,
    device: Res<RenderDevice>,
    mut extracted_cache: ResMut<ExtractedComponentCache>,
) {
    if entity_query.is_empty() {
        return;
    }

    for (id, spritesheet, group) in &instancing_groups {
        // TODO handle empty groups somehow
        assert!(!group.entities.is_empty());

        let mut instancing_data = vec![];

        for entity in &group.entities {
            // TODO handle entity removal without group update somehow
            let (transform, instance) = entity_query.get(*entity).unwrap();

            instancing_data.push(SpriteInstanceData {
                i_position: transform.translation,
                i_scale: transform.scale.xy(),
                i_tex_index: instance.texture_index,
            });
        }

        let buffer = extracted_cache.update_instancing_buffer(
            id,
            device.as_ref(),
            queue.as_ref(),
            &instancing_data,
        );
        let spritesheet =
            extracted_cache.update_instanced_spritesheet(id, device.as_ref(), spritesheet);

        commands.get_or_spawn(id).insert((buffer, spritesheet));
    }
}
