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

#[derive(Component, Debug)]
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

/// This plugin is generic over different types of aggregator that you define.
/// The default implementation is for Scenes using DescendantRootPosition::Scene
pub struct ArmatureLoaderPlugin<T: ArmatureLoader + Component> {
	pub descendant_root_position: DescendantRootPosition,
	pub(crate) _phantom_data: PhantomData<T>,
}

impl<T: ArmatureLoader + Component> ArmatureLoaderPlugin<T> {
	pub fn new(descendant_root_position: DescendantRootPosition) -> Self {
		Self {
			descendant_root_position,
			..default()
		}
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum DescendantRootPosition {
	/// Scenes are starting from a child that does not explicitly says that it's
	/// the root of a scene, nor is it's discoverable from another component,
	/// so this option will search through all children, finding the first
	/// grand-child that matches the name of the
	///
	#[default]
	Scene,
	/// Use the entity marked directly
	Direct,
	/// You can pass an entity directly to use as the search root
	Fixed(Entity),
}

impl<T: ArmatureLoader + Component> Default for ArmatureLoaderPlugin<T> {
	fn default() -> Self {
		Self {
			descendant_root_position: DescendantRootPosition::default(),
			_phantom_data: PhantomData::default(),
		}
	}
}

#[derive(Resource, Debug)]
pub(crate) struct DescendantCollectorSettings<T: ArmatureLoader + Component> {
	pub descendant_root_position: DescendantRootPosition,
	pub(crate) _phantom_data: PhantomData<T>,
}

impl<T: ArmatureLoader + Component> Plugin for ArmatureLoaderPlugin<T> {
	fn build(&self, app: &mut App) {
		app.insert_resource(DescendantCollectorSettings::<T> {
			descendant_root_position: self.descendant_root_position.clone(),
			_phantom_data: PhantomData::default(),
		});
		app.add_systems(PostUpdate, insert_armature_after_load::<T>);
	}
}

fn insert_armature_after_load<T: ArmatureLoader + Component>(
	mut commands: Commands,
	settings: Res<DescendantCollectorSettings<T>>,
	scenes_added: Query<(Entity, &SceneInstance), (Without<T>, Added<SceneInstance>)>,
	o_named_query: Query<(Entity, Option<&Name>, Option<&Children>)>,
) {
	for scenes_just_added in scenes_added.iter() {
		let armature_map_target = scenes_just_added.0;
		let armature_source_root_opt = match settings.descendant_root_position {
			DescendantRootPosition::Scene => find_named_grandchild(
				armature_map_target,
				&o_named_query,
				T::get_root_entity_name(),
			),
			DescendantRootPosition::Direct => Some(armature_map_target),
			DescendantRootPosition::Fixed(entity) => Some(entity),
		};

		let armature_source_root = armature_source_root_opt.expect(&format!(
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
