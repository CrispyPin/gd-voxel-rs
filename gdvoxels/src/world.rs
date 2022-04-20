use std::collections::HashMap;
use gdnative::prelude::*;

use crate::common::*;
use crate::chunk::*;

const CHUNK_FP: &str = "res://addons/voxel-engine/Chunk.tscn";

#[derive(NativeClass)]
#[inherit(Node)]
pub struct VoxelWorld {
	chunks: HashMap<(i32, i32, i32), Ref<ChunkNodeType, Shared>>,
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
			load_distance: 2,
			player_pos: Vector3::ZERO,
			chunk_resource,
		}
	}

	#[export]
	fn _ready(&mut self, owner: &Node) {
		self.load_near(owner);
	}

	/// load chunks around player pos
	fn load_near(&mut self, owner: &Node) {
		let center_chunk = chunk_pos(self.player_pos);
		let radius = self.load_distance as i32;
		let range = -radius..(radius + 1);
		for x in range.clone() {
			 for y in range.clone() {
				for z in range.clone() {
					let loc = center_chunk + ivec3(x, y, z);
					self.load_or_generate(owner, loc);
				}
			 } 
		}
	}

	/// if chunk at loc is not already loaded, generate a new one
	/// (todo) load from disk if it exists instead of generating
	fn load_or_generate(&mut self, owner: &Node, loc: Vector3) {
		if self.chunk_is_loaded(loc) {
			return;
		}
		self.generate(owner, loc);
	}

	fn generate(&mut self, owner: &Node, loc: Vector3) {
		let new_chunk = unsafe {
			self.chunk_resource
				.assume_safe()
				.instance(0)
				.unwrap()
				.assume_safe()
				.cast::<ChunkNodeType>()
				.unwrap()
		};
		new_chunk.set_name(format!("Chunk{:?}", key(loc)));
		new_chunk.set_translation(loc * WIDTH as f32);

		let chunk_ref = unsafe { new_chunk.assume_shared() };
		self.chunks.insert(key(loc), chunk_ref);

		owner.add_child(new_chunk, false);
	}

	fn chunk_is_loaded(&self, loc: Vector3) -> bool {
		self.chunks.contains_key(&key(loc))
	}
}

/// convert Vector3 to i32 tuple to use as a key in chunk array
fn key(loc: Vector3) -> (i32, i32, i32) {
	(loc.x as i32, loc.y as i32, loc.z as i32)
}
