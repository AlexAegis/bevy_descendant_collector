# [bevy\_descendant\_collector](https://github.com/AlexAegis/bevy_descendant_collector)

[![crates.io](https://img.shields.io/crates/v/bevy_descendant_collector.svg)](https://crates.io/crates/bevy_descendant_collector)
[![ci](https://github.com/AlexAegis/bevy_descendant_collector/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexAegis/bevy_descendant_collector/actions/workflows/ci.yml)

This crate lets you map a complex entity tree onto a single component as long
as those entities (and their ancestors) are named.

This is especially useful for scenes spawned from GLTF models since you'd need
complex queries to retrieve deep entities from the scene hierarchy.

## Example

Running the example:

```sh
cargo run -p bevy_descendant_collector --example turret --features example
```

Imagine you have a [model file](/crates/bevy_descendant_collector/assets/models/simple_turret.blend) with the
following structure in Blender:

```sh
Collection
├── Armature
│   ├── Bone.Root
│   │   └── Bone.Neck
│   │       └── Bone.Head
│   └── Bone.Head.IK_CTRL
```

You can define a component like this:

```rs
#[derive(Component, EntityCollectorTarget)]
#[name_path("Armature")]
pub struct MyTurretArmature {
	#[name_path("Bone.Root")]
	pub base: Entity,
	#[name_path("Bone.Root", "Bone.Neck")]
	pub neck: Entity,
	#[name_path("Bone.Root", "Bone.Neck", "Bone.Head")]
	pub head: Entity,
	#[name_path("Bone.Head.IK_CTRL")]
	pub head_ik_ctrl: Entity,
}
```

Then, you need to add a Plugin, this plugin will handle the insertion of this
component:

> Each different `EntityCollectorTarget` needs its own plugin added!

In this example this is a GLTF model, so I want the root entity to be resolved
as a scene. `HierarchyRootPosition::Scene` tells the plugin to find the root
of the hierarchy based on `#[name_path("Armature")]` among the grandchildren
of the entity where I want this `MyTurretArmature` to be inserted into.

```rs
app.add_plugins(DescendantCollectorPlugin::<MyTurretArmature>::new(HierarchyRootPosition::Scene));
```

And lastly, I need to identify the entities where I want `MyTurretArmature` to
be inserted into! To do this, when you spawn this scene, add the target
component to your entity.

```rs
fn spawn_turret(mut commands: Commands, turret_model_assets: Res<TurretModelAssets>) {
	commands.spawn((
		SceneBundle {
			scene: turret_model_assets.turret_model.clone(),
			..default()
		},
		DescendantCollectorTarget::<MyTurretArmature>::default(),
	));
}
```

## Expanding the proc macro

> In case you want to inspect the output of the proc macro.

If you haven't installed `cargo-expand` yet.

```sh
cargo install cargo-expand
```

```sh
cargo expand --example turret
```

## Bevy Compatibility Table

| Bevy | bevy\_descendant\_collector |
| ---- | ------------------------- |
| 0.15 | 0.3                       |
| 0.14 | 0.2                       |
| 0.13 | 0.1                       |
