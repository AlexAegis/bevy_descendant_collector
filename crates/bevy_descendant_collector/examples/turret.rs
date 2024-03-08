use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_descendant_collector::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::InspectorOptions;

#[derive(Default, AssetCollection, Resource)]
pub struct TurretModelAssets {
	#[asset(path = "models/simple_turret.glb#Scene0")]
	pub turret_model: Handle<Scene>,
}

/// This struct will be populated from a loaded gltf scene, based on name paths.
#[derive(Component, Armature, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
#[armature_path("Armature")]
pub struct MyTurretArmature {
	#[armature_path()]
	pub root: Entity,
	#[armature_path("Bone")]
	pub base: Entity,
	#[armature_path("Bone", "Bone.Neck")]
	pub neck: Entity,
	#[armature_path("Bone", "Bone.Neck", "Bone.Head")]
	pub head: Entity,
	#[armature_path("Bone.Head.Target.IK")]
	pub target_ik: Entity,
}

#[derive(States, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Clone, Copy)]
enum ExampleAppState {
	#[default]
	Loading,
	Loaded,
}

fn main() {
	App::new()
		.init_state::<ExampleAppState>()
		.add_plugins((
			DefaultPlugins,
			WorldInspectorPlugin::new(),
			ArmatureLoaderPlugin::<MyTurretArmature>::new(DescendantRootPosition::Scene),
		))
		.register_type::<MyTurretArmature>()
		.add_loading_state(
			LoadingState::new(ExampleAppState::Loading)
				.load_collection::<TurretModelAssets>()
				.continue_to_state(ExampleAppState::Loaded),
		)
		.add_systems(Startup, spawn_example_scene)
		.add_systems(OnEnter(ExampleAppState::Loaded), spawn_turret)
		.run();
}

fn spawn_turret(mut commands: Commands, turret_model_assets: Res<TurretModelAssets>) {
	commands.spawn((
		SceneBundle {
			scene: turret_model_assets.turret_model.clone(),
			..default()
		},
		ArmatureTarget::<MyTurretArmature>::default(), // marking this entity that it needs an aggregator
	));
}

fn spawn_example_scene(mut commands: Commands) {
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(1., 4., 5.).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});

	commands.spawn(PointLightBundle {
		transform: Transform::from_xyz(2.0, 0.6, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}
