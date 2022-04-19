use std::collections::HashMap;
use gdnative::prelude::*;

use crate::common::*;
use crate::chunk::*;

const CHUNK_FP: &str = "res://addons/voxel-engine/Chunk.tscn";

#[derive(NativeClass)]
#[inherit(Node)]
pub struct VoxelWorld {
	chunks: HashMap<(i32, i32, i32), Chunk>,
	load_distance: u16,
	player_pos: Vector3,
	chunk_resource: Ref<PackedScene>,
}

#[methods]
impl VoxelWorld {
	fn new(_owner: &Node) -> Self {
		let chunk_resource = ResourceLoader::godot_singleton()
			.load(CHUNK_FP, "PackedScene", false)
			.unwrap()
			.cast::<PackedScene>()
			.unwrap();
		
		Self {
			chunks: HashMap::new(),
			load_distance: 8,
			player_pos: Vector3::ZERO,
			chunk_resource,
		}
	}
}
