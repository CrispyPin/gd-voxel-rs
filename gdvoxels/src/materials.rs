use std::collections::HashMap;

use gdnative::prelude::*;
use gdnative::api::ShaderMaterial;

use crate::common::*;


const DEFAULT_PATH: &str = "res://addons/voxel-engine/materials/default.material";

pub struct VoxelMaterials {
	mats: HashMap<Voxel, Ref<ShaderMaterial, Shared>>,
	default: Ref<ShaderMaterial, Shared>,
}

impl VoxelMaterials {
	pub fn new() -> Self{
		let mut instance = Self {
			mats: HashMap::new(),
			default: load_mat_unsafe(DEFAULT_PATH),
		};
		instance.load();
		instance
	}

	fn load(&mut self) {
		let resource_loader = ResourceLoader::godot_singleton();
		for v in 0..256 {
			let voxel = v as Voxel;
			let path = format!("res://addons/voxel-engine/materials/voxels/{}.material", voxel.name());
			if resource_loader.exists(&path, "ShaderMaterial") {
				let mat = resource_loader
					.load(&path, "ShaderMaterial", false)
					.unwrap()
					.cast::<ShaderMaterial>()
					.unwrap();
				self.mats.insert(voxel, mat);
				godot_print!("loaded material: {}", &path);
			}
		}
	}

	pub fn get(&self, voxel: Voxel) -> Ref<ShaderMaterial, Shared> {
		if self.mats.contains_key(&voxel) {
			return self.mats.get(&voxel).unwrap().clone();
		}
		self.default.clone()
	}
}

fn load_mat_unsafe(path: &str) -> Ref<ShaderMaterial, Shared> {
	ResourceLoader::godot_singleton()
		.load(path, "ShaderMaterial", false)
		.unwrap()
		.cast::<ShaderMaterial>()
		.unwrap()
}
