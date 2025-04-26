use bevy::prelude::*;
use bevy_descendant_collector::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

/// This struct will be populated from a loaded gltf scene, based on name paths.
#[derive(Component, EntityCollectorTarget, Reflect, InspectorOptions)]
#[require(Name::new("Turret"))]
#[reflect(Component, InspectorOptions)]
#[name_path("Armature")] // This is only used when the root has to be automatically discovered, like for scenes
pub struct MyTurretArmature {
	#[name_path()]
	pub root: Entity,
	#[name_path("Bone.Root")]
	pub base: Entity,
	#[name_path("Bone.Root", "Bone.Neck")]
	pub neck: Entity,
	#[name_path("Bone.Root", "Bone.Neck", "Bone.Head")]
	pub head: Entity,
	#[name_path("Bone.Head.IK_CTRL")]
	pub head_ik_ctrl: Entity,
}

fn main() -> AppExit {
	App::new()
		.add_plugins((
			DefaultPlugins,
			EguiPlugin {
				enable_multipass_for_primary_context: true,
			},
			WorldInspectorPlugin::new(),
			DescendantCollectorPlugin::<MyTurretArmature>::new(HierarchyRootPosition::Scene),
		))
		.register_type::<MyTurretArmature>()
		.add_systems(Startup, setup)
		.run()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		SceneRoot(
			asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/simple_turret.glb")),
		),
		DescendantCollectorTarget::<MyTurretArmature>::default(), // marking this entity that it needs an accumulator
	));

	commands.spawn((
		Camera3d::default(),
		Transform::from_xyz(1., 4., 5.).looking_at(Vec3::ZERO, Vec3::Y),
	));

	commands.spawn((
		PointLight::default(),
		Transform::from_xyz(2.0, 0.6, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
	));
}
