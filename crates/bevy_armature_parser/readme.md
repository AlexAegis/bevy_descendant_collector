# bevy_armature_parser

This crate lets you load gltf models and parse their skeletons based on
conventional entity names. The goal is to collect all relevant parts of a
loaded model and hydrate it with components like IK constraits.

## Examples 

```sh
cargo run -p bevy_armature_parser --example turret
```

## Expanding the proc macro

If you haven't installed `cargo-expand` yet.

```sh
cargo install cargo-expand
```

```sh
 cargo expand --example turret
```