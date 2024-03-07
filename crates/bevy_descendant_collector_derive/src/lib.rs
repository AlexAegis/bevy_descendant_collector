use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Meta};

fn read_attribute(attrs: Vec<Attribute>, attribute_name: &str) -> Option<TokenStream> {
	let armature_path_attr = attrs
		.iter()
		.find(|attr| attr.path().is_ident("armature_path"));

	if let Some(attr) = armature_path_attr {
		if let Meta::List(list) = &attr.meta {
			Some(list.tokens.clone())
		} else {
			panic!("{attribute_name} has the wrong type");
		}
	} else {
		None
	}
}

/// Inserts this component to freshly spawned scenes marked with a generic component
#[proc_macro_derive(Armature, attributes(armature_path))]
pub fn armature_path_map(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let derive_input = parse_macro_input!(input as DeriveInput);
	let struct_name = derive_input.ident;
	let generics = derive_input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let fields = match derive_input.data {
		syn::Data::Struct(ref data) => match &data.fields {
			syn::Fields::Named(fields) => &fields.named,
			_ => panic!("Only named fields are supported"),
		},
		_ => panic!("Only structs are supported"),
	};

	let armature_entry_name = read_attribute(derive_input.attrs, "armature_path").expect("no root armature_path is defined, define the name of the root element (The first element in blender in the exported collection)");

	let assignments = fields
		.iter()
		.filter_map(|field| {
			let armature_field_entry_name = read_attribute(field.attrs.clone(), "armature_path").expect("Field's that are not armature_path's are not premitted");
			let ident = field.ident.as_ref().expect("The variable must have a name!");
			Some((ident, armature_field_entry_name))
		})
		.map(|(ident, path)| {
			let error_msg = if path.is_empty() {
				format!("Root armature element not found for {struct_name}. Actual name paths are:\n")
			} else {
				format!("Named armature element not found for {struct_name} at {path}. Actual name paths are:\n")
			};
			quote! {
				#ident: bevy_descendant_collector::find_named_entity::find_named_entity(armature_source_root, &named_query, &vec![#path]).unwrap_or_else(|| {
					let named_entity_paths = bevy_descendant_collector::find_named_entity::collect_named_entity_paths(armature_source_root, &named_query);
					panic!("{} {:#?}", #error_msg, named_entity_paths);
				}),
			}
		})
		.collect::<Vec<_>>();

	proc_macro::TokenStream::from(quote! {
		impl #impl_generics bevy_descendant_collector::armature_loader::ArmatureLoader for #struct_name #ty_generics #where_clause {

			fn get_root_entity_name() -> &'static str {
				#armature_entry_name
			}

			fn insert_armature(
				commands: &mut Commands,
				armature_source_root: Entity,
				armature_map_target: Entity,
				named_query: &Query<(Entity, Option<&Name>, Option<&Children>)>) {
				let armature = MyTurretArmature {
					#(#assignments)*
				};
				commands.entity(armature_map_target).insert(armature);
			}
		}
	})
}
