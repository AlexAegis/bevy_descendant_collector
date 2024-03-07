use bevy::prelude::*;
use bevy_armature_parser::armature_loader::{ArmatureRegistry, ArmatureTarget};
use bevy_armature_parser_derive::Armature;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
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
enum MyTurretExampleState {
	#[default]
	Loading,
	Loaded,
}

#[derive(Component, Default, Debug)]
struct MyTurret;

fn main() {
	App::new()
		.add_plugins((DefaultPlugins, WorldInspectorPlugin::new()))
		.register_type::<MyTurretArmature>()
		.register_armature::<MyTurretArmature>()
		.init_state::<MyTurretExampleState>()
		.add_loading_state(
			LoadingState::new(MyTurretExampleState::Loading)
				.load_collection::<TurretModelAssets>()
				.continue_to_state(MyTurretExampleState::Loaded),
		)
		.add_systems(Startup, (setup_camera, setup_example_scene))
		.add_systems(OnEnter(MyTurretExampleState::Loaded), setup_turret)
		.run();
}

fn setup_turret(mut commands: Commands, turret_model_assets: Res<TurretModelAssets>) {
	commands.spawn((
		MyTurret,
		SceneBundle {
			scene: turret_model_assets.turret_model.clone(),
			..default()
		},
		ArmatureTarget::<MyTurretArmature>::default(),
	));
}

fn setup_camera(mut commands: Commands) {
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(1., 4., 5.).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}

fn setup_example_scene(mut commands: Commands, assets: ResMut<AssetServer>) {
	let box_handle = assets.add(Cuboid::new(1., 1., 1.).into());

	commands.spawn(PointLightBundle {
		transform: Transform::from_xyz(2.0, 0.6, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});

	commands.spawn(PbrBundle {
		mesh: box_handle,
		transform: Transform::from_xyz(0.0, -1.0, 0.0),
		material: assets.add(
			StandardMaterial {
				base_color: Color::GRAY,
				..Default::default()
			}
			.into(),
		),
		..default()
	});
}
