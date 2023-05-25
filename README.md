`bevy_sprite_instancing`
========================

[![crates.io](https://img.shields.io/crates/v/bevy_sprite_instancing)](https://crates.io/crates/bevy_sprite_instancing)
![License](https://img.shields.io/crates/l/bevy_sprite_instancing)

A plugin for Bevy to render lots of instanced sprites in a single draw call.

How does the plugin work?
-------------------------

Instead of going through all similar entity sprites one-by-one,
the plugin provides  `SpriteInstancingGroup`s which can contain
Entity ID's of these sprites. The plugin then collect all these sprites'
transforms and submits them to the GPU as one large buffer, which is then
used to draw all of them at once.

![Animated example](example.gif)

Features
--------

* Animated tiles
* Nice FPS when drawing lots of sprites (up to 1 million)

How do I use this?
------------------

For a quick example, see [this example](examples/simple.rs).

Here's a simple example of how to draw lots of sprites:

```rust
pub const ENTITY_COUNT: usize = 100000;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	// Create a camera
	commands.spawn(Camera2dBundle::default());

	// Load a spritesheet
	let spritesheet = InstancedSpritesheet {
		image: asset_server.load("textures/my_spritesheet.png"),
		// Size of this spritesheet in tiles
		width_tiles: 32,
		height_tiles: 32,
	};

	// Create an instancing group for the sprites
	let mut instancing_group = SpriteInstancingGroup {
		entities: HashSet::new(),
	};
	let group_id = commands.spawn_empty().id();

	// Spawn the sprites
	for _ in 0..ENTITY_COUNT {
		let position = Vec3::new(..., 0.0); // some random position
		let scale = Vec3::new(..., 1.0); // some random size
		let transform = ...; // create a transform from these
		let sprite = InstancedSprite {
			group_id,
			texture_index: 0
		};

		let entity = commands.spawn((transform, sprite)).id();
		instancing_group.entities.insert(entity);
	}

	// Attach the instancing group and its spritesheet
	commands.entity(group_id).insert((instancing_group, spritesheet));
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(InstancedSpriteRenderPlugin)
		.add_startup_system(setup)
		.run()
}
```


What I didn't yet implement
---------------------------

There're some high-level features I haven't yet implemented:

* View region culling: no need to submit transforms of sprites which aren't
	present on screen
* GPU animations: instead of submitting a `texture_index` for each sprite,
	maybe it would be a nice idea to submit `anim_start_index` and `anim_len`
	through a separate instancing buffer

... and some low-level ones:

* Ability to mark a `SpriteInstancingGroup` as static (i.e. entities do not
	move) to avoid re-submitting its instancing data each frame
