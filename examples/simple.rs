use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec3Swizzles,
    prelude::*,
    utils::HashSet,
};
use bevy_sprite_instancing::{
    InstancedSprite, InstancedSpriteRenderPlugin, InstancedSpritesheet, SpriteInstancingGroup,
};

pub const ENTITY_COUNT: usize = 100000;

fn random_transform(s_base: f32, s_mul: f32) -> Transform {
    let x = (rand::random::<f32>() - 0.5) * 2000.0;
    let y = (rand::random::<f32>() - 0.5) * 1000.0;
    let s = (rand::random::<f32>() - 0.5) * s_mul + s_base;

    Transform::from_translation(Vec3::new(x, y, 0.0)).with_scale(Vec3::new(s, s, 1.0))
}

fn random_delta() -> Vec3 {
    let dx = (rand::random::<f32>() - 0.5) * 20.0;
    let dy = (rand::random::<f32>() - 0.5) * 20.0;

    Vec3::new(dx, dy, 0.0)
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let image = asset_server.load("map0.png");
    let spritesheet0 = InstancedSpritesheet {
        image,
        width_tiles: 32,
        height_tiles: 32,
    };

    let mut instancing_group0 = SpriteInstancingGroup {
        entities: HashSet::new(),
    };

    let instancing_group0_id = commands.spawn_empty().id();

    for _ in 0..ENTITY_COUNT {
        let id = commands
            .spawn((
                random_transform(16.0, 10.0),
                InstancedSprite {
                    group_id: instancing_group0_id,
                    texture_index: 0,
                },
            ))
            .id();

        instancing_group0.entities.insert(id);
    }

    commands
        .entity(instancing_group0_id)
        .insert((instancing_group0, spritesheet0));
}

fn move_entities(mut query: Query<&mut Transform, With<InstancedSprite>>) {
    for mut transform in query.iter_mut() {
        transform.translation += random_delta();
    }
}

fn animate_entities(mut query: Query<&mut InstancedSprite>, time: Res<Time>) {
    let t = time.elapsed_seconds();
    let animation_step = (t * 10.0) as u32 % 5;

    for mut instance in query.iter_mut() {
        instance.texture_index = animation_step;
    }
}

fn handle_clicks(
    mut commands: Commands,
    entities: Query<(Entity, &Transform, &InstancedSprite)>,
    mut instance_groups: Query<&mut SpriteInstancingGroup>,
    window: Query<&Window>,
    mouse_button: Res<Input<MouseButton>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = window.single();
        let position = window.cursor_position().unwrap()
            - Vec2::new(window.width() / 2.0, window.height() / 2.0);

        for (entity, transform, instance) in &entities {
            let pos = transform.translation.xy();

            if pos.distance(position) < 10.0 {
                commands.entity(entity).despawn();
                instance_groups
                    .get_mut(instance.group_id)
                    .unwrap()
                    .entities
                    .remove(&entity);
            }
        }
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InstancedSpriteRenderPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(move_entities)
        .add_system(animate_entities)
        .add_system(handle_clicks)
        .run();
}
