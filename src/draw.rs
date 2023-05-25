use bevy::{
    ecs::system::{
        lifetimeless::{Read, SRes},
        Res,
    },
    render::{
        mesh::GpuBufferInfo,
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
    },
    sprite::SetMesh2dViewBindGroup,
};

use super::{
    extract::ExtractedSpriteInstancingBuffer, InstancedSpriteMesh, InstancedSpritesheetBindGroup,
};

pub struct DrawSpritesInstanced;

pub(super) type DrawSpritesInstancedCommands = (
    SetItemPipeline,
    // View uniform
    SetMesh2dViewBindGroup<0>,
    DrawSpritesInstanced,
);

impl<P: PhaseItem> RenderCommand<P> for DrawSpritesInstanced {
    type Param = SRes<InstancedSpriteMesh>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (
        Read<ExtractedSpriteInstancingBuffer>,
        Read<InstancedSpritesheetBindGroup>,
    );

    fn render<'w>(
        _item: &P,
        _view: (),
        (instancing_buffer, instancing_spritesheet): (
            &'w ExtractedSpriteInstancingBuffer,
            &'w InstancedSpritesheetBindGroup,
        ),
        instancing_mesh: Res<'w, InstancedSpriteMesh>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let instancing_mesh = instancing_mesh.into_inner();

        pass.set_vertex_buffer(0, instancing_mesh.quad.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instancing_buffer.device_buffer.slice(..));
        pass.set_bind_group(1, &instancing_spritesheet.bind_group, &[]);

        match &instancing_mesh.quad.buffer_info {
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instancing_buffer.length as u32);
            }
            _ => todo!(),
        }

        RenderCommandResult::Success
    }
}
