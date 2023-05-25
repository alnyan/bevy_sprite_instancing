use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindingResource, PipelineCache,
            SpecializedMeshPipelines,
        },
        renderer::RenderDevice,
        view::ExtractedView,
    },
    sprite::Mesh2dPipelineKey,
    utils::FloatOrd,
};

use super::{
    draw::DrawSpritesInstancedCommands,
    extract::{ExtractedInstancedSpritesheet, ExtractedSpriteInstancingBuffer},
    pipeline::InstancedSpritePipeline,
    InstancedSpriteMesh, InstancedSpritesheetBindGroup,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_instanced_spritesheets(
    mut commands: Commands,
    pipeline: Res<InstancedSpritePipeline>,
    images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
    spritesheet_query: Query<(Entity, &ExtractedInstancedSpritesheet)>,
) {
    for (id, spritesheet) in &spritesheet_query {
        let Some(spritesheet_image) = images.get(&spritesheet.image) else {
            continue;
        };

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("Instanced spritesheet bind group"),
            layout: &pipeline.spritesheet_uniform_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&spritesheet_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&spritesheet_image.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(
                        spritesheet.size_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        commands
            .get_or_spawn(id)
            .insert(InstancedSpritesheetBindGroup { bind_group });
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_instanced_sprites(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    instanced_entity_pipeline: Res<InstancedSpritePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancedSpritePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    entity_instancing_mesh: Res<InstancedSpriteMesh>,
    entity_instancing_groups: Query<
        Entity,
        (
            With<InstancedSpritesheetBindGroup>,
            With<ExtractedSpriteInstancingBuffer>,
        ),
    >,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &ExtractedView)>,
) {
    let layout = &entity_instancing_mesh.quad.layout;

    for (mut transparent_phase, view) in &mut views {
        let draw_function = transparent_draw_functions
            .read()
            .id::<DrawSpritesInstancedCommands>();

        let key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr)
            | Mesh2dPipelineKey::from_primitive_topology(
                entity_instancing_mesh.quad.primitive_topology,
            );

        let pipeline = pipelines
            .specialize(&pipeline_cache, &instanced_entity_pipeline, key, layout)
            .unwrap();

        for entity in &entity_instancing_groups {
            transparent_phase.add(Transparent2d {
                sort_key: FloatOrd(0.0),
                entity,
                pipeline,
                draw_function,
                batch_range: None,
            });
        }
    }
}
