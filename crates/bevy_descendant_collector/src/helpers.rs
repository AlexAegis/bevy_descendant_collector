use bevy_ecs::{entity::Entity, hierarchy::Children, name::Name, system::Query};

pub fn find_named_entity(
	root_entity: Entity,
	named_query: &Query<(Entity, Option<&Name>, Option<&Children>)>,
	search_terms: &[&str],
) -> Option<Entity> {
	if search_terms.is_empty() {
		return Some(root_entity);
	}

	let children_option = named_query.get(root_entity).ok()?.2?;

	let next_named_child = children_option
		.iter()
		.flat_map(|child_entity| named_query.get(*child_entity))
		.filter_map(|(entity, name_opt, _children)| {
			name_opt.and_then(|name| {
				if name.as_str() == search_terms[0] {
					Some(entity)
				} else {
					None
				}
			})
		})
		.next()?;

	find_named_entity(next_named_child, named_query, &search_terms[1..])
}

pub fn find_named_grandchild(
	root_entity: Entity,
	named_query: &Query<(Entity, Option<&Name>, Option<&Children>)>,
	target_name: &str,
) -> Option<Entity> {
	named_query
		.get(root_entity)
		.iter()
		.flat_map(|(_e, _name, immediate_children)| immediate_children)
		.flat_map(|children| children.iter())
		.flat_map(|child| named_query.get(*child))
		.flat_map(|(_e, _child_name, grand_children)| grand_children)
		.flat_map(|grand_children| grand_children.iter())
		.flat_map(|grand_child| named_query.get(*grand_child))
		.find(|(_e, grand_child_name, _ggch)| {
			grand_child_name.is_some_and(|name| target_name == name.to_string())
		})
		.map(|(entity, _grand_child_name, _ggch)| entity)
}

pub fn collect_named_entity_paths(
	source_root: Entity,
	named_query: &Query<(Entity, Option<&Name>, Option<&Children>)>,
) -> Vec<Vec<String>> {
	let mut paths = Vec::new();
	if let Ok((_, root_name_opt, children)) = named_query.get(source_root) {
		let root_path = match root_name_opt {
			Some(root_name) => vec![root_name.as_str().to_string()],
			None => vec![],
		};

		// If the root has children, start the recursion
		if let Some(children) = children {
			for child in children.iter() {
				collect_paths_internal(*child, &root_path, named_query, &mut paths);
			}
		} else if !root_path.is_empty() {
			// If the root has no children but has a name, it's a valid path by itself
			paths.push(root_path);
		}
	}
	paths
}

fn collect_paths_internal(
	entity: Entity,
	current_path: &[String],
	named_query: &Query<(Entity, Option<&Name>, Option<&Children>)>,
	paths: &mut Vec<Vec<String>>,
) {
	// Skip entities without a name
	if let Ok((_, Some(name), children)) = named_query.get(entity) {
		let mut new_path = current_path.to_vec();
		new_path.push(name.as_str().to_string());

		if let Some(children) = children {
			if children.is_empty() {
				// If the entity has a name but no children, it's the end of a path
				paths.push(new_path);
			} else {
				for child in children.iter() {
					collect_paths_internal(*child, &new_path, named_query, paths);
				}
			}
		} else {
			// If the entity has a name but does not have a Children component, it's a leaf node
			paths.push(new_path);
		}
	}
}

#[cfg(test)]
mod test {
	use bevy::prelude::*;

	use crate::helpers::{collect_named_entity_paths, find_named_entity};

	#[derive(Resource, Default)]
	struct EntitySubject {
		root: Option<Entity>,
	}

	fn setup_entities(mut commands: Commands, mut entity_subject: ResMut<EntitySubject>) {
		let root = commands
			.spawn(())
			.with_children(|root_entity| {
				root_entity
					.spawn(Name::new("foo1"))
					.with_children(|foo_entity| {
						foo_entity
							.spawn(Name::new("bar"))
							.with_children(|bar_entity| {
								bar_entity.spawn(Name::new("baz"));
								bar_entity.spawn(Name::new("baz2"));
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

	fn assert_find_baz2(
		entity_subject: Res<EntitySubject>,
		named_query: Query<(Entity, Option<&Name>, Option<&Children>)>,
	) {
		let root_entity = entity_subject.root.expect("root should be defined");

		let my_baz = find_named_entity(root_entity, &named_query, &["foo2", "bar", "baz"]);

		assert!(my_baz.is_some());

		let (_my_baz_entity, my_baz_name, _my_baz_children) = named_query
			.get(my_baz.expect("should be found"))
			.expect("should be found");

		assert_eq!(
			my_baz_name.expect("baz should have a name").to_string(),
			"baz"
		);
	}

	#[test]
	fn test_find_named_entity() {
		let mut app = App::new();
		app.init_resource::<EntitySubject>();
		app.add_systems(Startup, setup_entities);
		app.add_systems(Update, assert_find_baz2);
		app.update(); // Startup
		app.update(); // Update once
	}

	fn assert_collect_named_descendants(
		entity_subject: Res<EntitySubject>,
		named_query: Query<(Entity, Option<&Name>, Option<&Children>)>,
	) {
		let root_entity = entity_subject.root.expect("root should be defined");

		let named_entity_paths = collect_named_entity_paths(root_entity, &named_query);

		assert_eq!(
			named_entity_paths,
			vec![
				vec!["foo1", "bar", "baz"],
				vec!["foo1", "bar", "baz2"],
				vec!["foo2", "bar", "baz"],
				vec!["foo2", "bar", "baz2"],
				vec!["foo2", "bar", "baz3"]
			]
		);
	}

	#[test]
	fn test_collect_named_descendants() {
		let mut app = App::new();
		app.init_resource::<EntitySubject>();
		app.add_systems(Startup, setup_entities);
		app.add_systems(Update, assert_collect_named_descendants);
		app.update(); // Startup
		app.update(); // Update once
	}
}
