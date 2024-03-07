use std::marker::PhantomData;

use bevy::{prelude::*, scene::SceneInstance};

use crate::find_named_entity::find_named_grandchild;

/// Implemented by the Armature derive trait, used by the ArmatureLoaderPlugin
pub trait ArmatureLoader {
	fn get_root_entity_name() -> &'static str;

	/// Should be executed only once, after the scene is loaded
	/// armature_source_root is the entity where the resolution of name paths begins
	/// armature_map_target is where the created Armature component will be inserted to
	fn insert_armature(
		commands: &mut Commands,
		armature_source_root: Entity,
		armature_map_target: Entity,
		named_query: &Query<(Entity, Option<&Name>, Option<&Children>)>,
	);
}

#[derive(Component)]
pub struct ArmatureTarget<T: ArmatureLoader + Component> {
	_phantom_data: PhantomData<T>,
}

impl<T: ArmatureLoader + Component> Default for ArmatureTarget<T> {
	fn default() -> Self {
		Self {
			_phantom_data: Default::default(),
		}
	}
}

impl<T: ArmatureLoader + Component> ArmatureTarget<T> {
	pub fn new() -> Self {
		Self { ..default() }
	}
}

/// This plugin is generic over different types of armatures that you define.
pub struct ArmatureLoaderPlugin<T: ArmatureLoader + Component> {
	_phantom_data: PhantomData<T>,
}

impl<T: ArmatureLoader + Component> Default for ArmatureLoaderPlugin<T> {
	fn default() -> Self {
		Self { ..default() }
	}
}

impl<T: ArmatureLoader + Component> Plugin for ArmatureLoaderPlugin<T> {
	fn build(&self, app: &mut App) {
		app.add_systems(PostUpdate, insert_armature_after_load::<T>);
	}
}

pub trait ArmatureRegistry {
	fn register_armature<T: ArmatureLoader + Component>(&mut self) -> &mut Self;
}

impl ArmatureRegistry for App {
	fn register_armature<T: ArmatureLoader + Component>(&mut self) -> &mut Self {
		// self.add_plugins(ArmatureLoaderPlugin::<T>::default())
		self.add_systems(PostUpdate, insert_armature_after_load::<T>)
	}
}

fn insert_armature_after_load<T: ArmatureLoader + Component>(
	mut commands: Commands,
	scenes_added: Query<(Entity, &SceneInstance), (Without<T>, Added<SceneInstance>)>,
	o_named_query: Query<(Entity, Option<&Name>, Option<&Children>)>,
) {
	for scenes_just_added in scenes_added.iter() {
		let armature_map_target = scenes_just_added.0;
		let armature_source_root = find_named_grandchild(
			armature_map_target,
			&o_named_query,
			T::get_root_entity_name(),
		)
		.expect(&format!(
			"Root of armature not found for {}",
			T::get_root_entity_name()
		));

		T::insert_armature(
			&mut commands,
			armature_source_root,
			armature_map_target,
			&o_named_query,
		);
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn test_armature_loader() {}
}
