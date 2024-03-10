use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{find_named_entity, helpers::find_named_grandchild};

/// Implemented by the Armature derive trait, used by the DescendantCollectorPlugin
pub trait DescendantLoader {
	fn get_root_entity_name() -> &'static str;

	/// Should be executed only once, after the scene is loaded
	/// entity_source_root is the entity where the resolution of name paths begins
	/// entity_map_target is where the created Armature component will be inserted to
	fn collect_descendants(
		commands: &mut Commands,
		entity_source_root: Entity,
		entity_map_target: Entity,
		named_query: &Query<(Entity, Option<&Name>, Option<&Children>)>,
	);
}

#[derive(Component, Debug)]
pub struct DescendantCollectorTarget<T: DescendantLoader + Component> {
	_phantom_data: PhantomData<T>,
}

impl<T: DescendantLoader + Component> Default for DescendantCollectorTarget<T> {
	fn default() -> Self {
		Self {
			_phantom_data: Default::default(),
		}
	}
}

impl<T: DescendantLoader + Component> DescendantCollectorTarget<T> {
	pub fn new() -> Self {
		Self { ..default() }
	}
}

/// This plugin is generic over different types of aggregator that you define.
/// The default implementation is for Scenes using DescendantRootPosition::Scene
pub struct DescendantCollectorPlugin<T: DescendantLoader + Component> {
	pub descendant_root_position: DescendantRootPosition,
	pub(crate) _phantom_data: PhantomData<T>,
}

impl<T: DescendantLoader + Component> DescendantCollectorPlugin<T> {
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
	Child,
	/// Use the entity marked directly
	Direct,
	/// You can pass an entity directly to use as the search root
	Fixed(Entity),
}

impl<T: DescendantLoader + Component> Default for DescendantCollectorPlugin<T> {
	fn default() -> Self {
		Self {
			descendant_root_position: DescendantRootPosition::default(),
			_phantom_data: PhantomData,
		}
	}
}

#[derive(Resource, Debug)]
pub(crate) struct DescendantCollectorSettings<T: DescendantLoader + Component> {
	pub descendant_root_position: DescendantRootPosition,
	pub(crate) _phantom_data: PhantomData<T>,
}

impl<T: DescendantLoader + Component> Plugin for DescendantCollectorPlugin<T> {
	fn build(&self, app: &mut App) {
		app.insert_resource(DescendantCollectorSettings::<T> {
			descendant_root_position: self.descendant_root_position,
			_phantom_data: PhantomData,
		});
		app.add_systems(PostUpdate, collect_descendants_after_load::<T>);
	}
}

fn collect_descendants_after_load<T: DescendantLoader + Component>(
	mut commands: Commands,
	settings: Res<DescendantCollectorSettings<T>>,
	collector_targets_added: Query<
		(Entity, &DescendantCollectorTarget<T>),
		(Without<T>, Added<DescendantCollectorTarget<T>>),
	>,
	name_query: Query<(Entity, Option<&Name>, Option<&Children>)>,
) {
	for scenes_just_added in collector_targets_added.iter() {
		let entity_map_target = scenes_just_added.0;

		let root_entity_name = T::get_root_entity_name();

		let entity_source_root_opt = match settings.descendant_root_position {
			DescendantRootPosition::Scene => {
				find_named_grandchild(entity_map_target, &name_query, root_entity_name)
			}
			DescendantRootPosition::Child => {
				find_named_entity(entity_map_target, &name_query, &[root_entity_name])
			}
			DescendantRootPosition::Direct => Some(entity_map_target),
			DescendantRootPosition::Fixed(entity) => Some(entity),
		};

		let entity_source_root = entity_source_root_opt
			.unwrap_or_else(|| panic!("Root of armature not found for {}", root_entity_name));

		T::collect_descendants(
			&mut commands,
			entity_source_root,
			entity_map_target,
			&name_query,
		);
	}
}

#[cfg(test)]
mod test {
	use bevy::prelude::*;

	use crate as bevy_descendant_collector;
	use crate::*;

	#[derive(Resource, Default)]
	struct EntitySubject {
		root: Option<Entity>,
		baz: Option<Entity>,
		baz2: Option<Entity>,
	}

	#[derive(Component, EntityCollectorTarget)]
	#[name_path("foo1")]
	struct EntityAccumulator {
		#[name_path("bar", "baz")]
		baz: Entity,
		#[name_path("bar", "baz2")]
		baz2: Entity,
	}

	fn setup_entities(mut commands: Commands, mut entity_subject: ResMut<EntitySubject>) {
		println!("setup_entities");
		let root = commands
			.spawn(DescendantCollectorTarget::<EntityAccumulator>::new())
			.with_children(|root_entity| {
				root_entity
					.spawn(Name::new("foo1"))
					.with_children(|foo_entity| {
						foo_entity
							.spawn(Name::new("bar"))
							.with_children(|bar_entity| {
								// Immediately adding them to a resource to be used for assertion later.
								entity_subject.baz = Some(bar_entity.spawn(Name::new("baz")).id());
								entity_subject.baz2 =
									Some(bar_entity.spawn(Name::new("baz2")).id());
								bar_entity.spawn(());
							});
					});

				root_entity
					.spawn(Name::new("foo2"))
					.with_children(|foo_entity| {
						foo_entity
							.spawn(Name::new("bar"))
							.with_children(|bar_entity| {
								bar_entity.spawn(Name::new("baz"));
								bar_entity.spawn(Name::new("baz2"));
								bar_entity.spawn(Name::new("baz3"));
							});
					});
			})
			.id();

		entity_subject.root = Some(root);
	}

	fn assert_accumulator(
		entity_subject: Res<EntitySubject>,
		named_query: Query<(Entity, &EntityAccumulator)>,
	) {
		let spawned_root_entity = entity_subject.root.expect("root should be defined");
		let spawned_baz_entity = entity_subject.baz.expect("baz should be defined");
		let spawned_baz2_entity = entity_subject.baz2.expect("baz2 should be defined");

		let (retrieved_root_entity, retrieved_accumulator) = named_query.get_single().expect("asd");
		println!("assert_accumulator");

		assert_eq!(retrieved_root_entity, spawned_root_entity);
		assert_eq!(retrieved_accumulator.baz, spawned_baz_entity);
		assert_eq!(retrieved_accumulator.baz2, spawned_baz2_entity);
	}

	#[test]
	fn test_entity_collector_from_child() {
		let mut app = App::new();
		app.init_resource::<EntitySubject>();
		app.add_plugins(DescendantCollectorPlugin::<EntityAccumulator>::new(
			DescendantRootPosition::Child,
		));
		app.add_systems(Startup, setup_entities);
		app.update(); // Startup
		app.add_systems(Update, assert_accumulator);
		app.update(); // Assert Collector
	}
}
