use std::collections::HashMap;
use gdnative::prelude::*;

use crate::common::*;
use crate::chunk::*;

const CHUNK_FP: &str = "res://addons/voxel-engine/Chunk.tscn";

#[derive(NativeClass)]
#[inherit(Node)]
pub struct VoxelWorld {
	#[property]
	load_distance: u16,
	#[property]
	player_pos: Vector3,
	chunks: HashMap<(i32, i32, i32), Ref<ChunkNodeType, Shared>>,
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

	#[export]
	fn _process(&mut self, owner: &Node, _delta: f32) {
		// let input = Input::godot_singleton();
		// if input.is_action_just_pressed("f2", false) {
			self.load_near(owner);
		// }
	}

	/// casts ray through world, counts as a miss if an unloaded chunk is entered
	/// max_dist is clamped to 0.001..4096
	#[export]
	fn cast_ray(&mut self, _owner: &Node, start_pos: Vector3, direction: Vector3, max_dist: f32) -> RayResult {
		let max_dist = max_dist.clamp(0.001, 4096.0);
		let direction = direction.normalized();
		
		let mut loc = chunk_loc(start_pos);
		let mut ray_len = 0.0;
		let mut ray_pos = start_pos;

		let mut steps = 0;
		while steps < 256 {
			steps += 1;
			if self.chunk_is_loaded(loc) {
				let chunk = self.get_chunk_unsafe(loc);
				let mut inner_ray =  chunk.map(
					|chunk, _owner| -> RayResult {
						chunk.cast_ray(local_pos(ray_pos), direction, max_dist - ray_len)
				}).unwrap();
				ray_len += inner_ray.distance;

				if inner_ray.hit {
					inner_ray.distance = ray_len;
					return inner_ray;
				}
				loc = chunk_loc(inner_ray.pos);
				ray_pos = inner_ray.pos;
			}
			else {
				return RayResult::miss(start_pos + direction * max_dist, max_dist);
			}
		}
		RayResult::miss(start_pos + direction * max_dist, max_dist)
	}

	#[export]
	fn set_voxel(&mut self, _owner: &Node, pos: Vector3, voxel: Voxel) {
		let chunk = self.get_chunk_unsafe(chunk_loc(pos));
		chunk.map_mut(|chunk, _owner| {
			chunk.set_voxel(local_pos(pos), voxel);
		}).unwrap();

	}

	/// load chunks around player pos
	fn load_near(&mut self, owner: &Node) {
		let center_chunk = chunk_loc(self.player_pos);
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
		self.create_chunk(owner, loc);
	}

	fn create_chunk(&mut self, owner: &Node, loc: Vector3) {
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

	fn get_chunk(&mut self, loc: Vector3) -> Option<TInstance<Chunk>> {
		if self.chunk_is_loaded(loc) {
			return Some(self.get_chunk_unsafe(loc));
		}
		None
	}

	fn get_chunk_unsafe(&mut self, loc: Vector3) -> TInstance<Chunk> {
		unsafe {
			self.chunks.get(&key(loc))
			.unwrap()
			.assume_safe()
			.cast_instance::<Chunk>()
			.unwrap()
		}
	}
}

/// convert Vector3 to i32 tuple to use as a key in chunk array
#[inline]
fn key(loc: Vector3) -> (i32, i32, i32) {
	(loc.x as i32, loc.y as i32, loc.z as i32)
}
