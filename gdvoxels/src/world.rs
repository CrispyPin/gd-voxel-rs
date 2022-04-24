use std::collections::HashMap;
use std::time::Instant;
use gdnative::api::MeshInstance;
use gdnative::prelude::*;

use crate::common::*;
use crate::chunk::*;
use crate::materials::*;

const CHUNK_PATH: &str = "res://addons/voxel-engine/Chunk.tscn";

type Loc = (i32, i32, i32);

enum MeshUpdate {
	Full(Loc),
	Partial(Loc, Vector3),
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct VoxelWorld {
	#[property]
	load_distance: u16,
	#[property]
	auto_load: bool,
	#[property]
	player_pos: Vector3,
	chunks: HashMap<Loc, Chunk>,
	chunk_update_queue: Vec<MeshUpdate>,
	chunk_resource: Ref<PackedScene>,
	materials: VoxelMaterials,
}

#[methods]
impl VoxelWorld {
	fn new(_owner: &Node) -> Self {
		let chunk_resource = ResourceLoader::godot_singleton()
			.load(CHUNK_PATH, "PackedScene", false)
			.unwrap()
			.cast::<PackedScene>()
			.unwrap();
		
		Self {
			chunks: HashMap::new(),
			chunk_update_queue: Vec::new(),
			materials: VoxelMaterials::new(),
			load_distance: 2,
			auto_load: true,
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
		if self.auto_load {
			self.load_near(owner);
		}
		
		for op in self.chunk_update_queue.iter() {
			let start_time = Instant::now();
			match op {
				MeshUpdate::Full(loc) => self.chunks.get_mut(loc).unwrap().mesh_full(&self.materials),
				MeshUpdate::Partial(loc, pos) => self.chunks.get_mut(loc).unwrap().mesh_partial(&self.materials, *pos),
			}
			godot_print!("remesh took: {}ms", start_time.elapsed().as_micros() as f64 / 1000.0);
		}
		self.chunk_update_queue.clear();
	}

	/// casts ray through world, sees unloaded chunks as empty
	/// max_len is clamped to 0.001..65536.0
	#[export]
	fn cast_ray(&mut self, owner: &Node, source: Vector3, dir: Vector3, max_len: f32) -> RayResult {
		let dir = dir.normalized();
		let max_len = max_len.clamp(0.001, 65536.0);
		let stepped_dir = step(0.0, dir);
		let mut ray_len = 0.0;
		
		while ray_len <= max_len {
			let ray_pos = source + dir * ray_len;
			let voxel = self.get_voxel(owner, ray_pos);
			if voxel != EMPTY {
				let normal = calc_normal(ray_pos);
				return RayResult::hit(ray_pos, normal, voxel, ray_len);
			}
			
			let pos_in_voxel = fract(fract(ray_pos) + Vector3::ONE);
			// distance "forward" along each axis to the next plane intersection
			let dist_to_next_plane = stepped_dir - pos_in_voxel;
			// distance along the ray to next plane intersection for each axis
			let deltas = dist_to_next_plane / dir;
			// move the smallest of these distances, so that no voxel is skipped
			ray_len += mincomp(deltas).max(0.0001);
		}
		RayResult::miss(source + dir * max_len, max_len)
	}

	#[export]
	fn set_voxel(&mut self, _owner: &Node, pos: Vector3, voxel: Voxel) {
		let loc = chunk_loc(pos);
		self.load_or_generate(_owner, loc);
		let chunk = self.get_chunk(loc);
		if chunk.is_some() {
			let chunk = chunk.unwrap();
			let pos = local_pos(pos);
			chunk.set_voxel(pos, voxel);
			self.chunk_update_queue.push(MeshUpdate::Partial(key(loc), pos));
		}
	}

	#[export]
	fn get_voxel(&mut self, _owner: &Node, pos: Vector3) -> Voxel{
		let loc = chunk_loc(pos);
		if let Some(chunk) = self.get_chunk(loc) {
			chunk.get_voxel(local_pos(pos))
		}
		else {
			EMPTY
		}
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
		let mesh = unsafe {
			self.chunk_resource
			.assume_safe()
			.instance(0)
			.unwrap()
			.assume_safe()
			.cast::<MeshInstance>()
			.unwrap()
		};
		let new_chunk = Chunk::new(loc * WIDTH_F);

		mesh.set_mesh(new_chunk.get_mesh());
		mesh.set_translation(loc * WIDTH_F);
		mesh.set_name(format!("Chunk{:?}", key(loc)));
		owner.add_child(mesh, false);
		
		self.chunks.insert(key(loc), new_chunk);
		self.chunk_update_queue.push(MeshUpdate::Full(key(loc)));
	}

	fn chunk_is_loaded(&self, loc: Vector3) -> bool {
		self.chunks.contains_key(&key(loc))
	}

	fn get_chunk(&mut self, loc: Vector3) -> Option<&mut Chunk> {
		self.chunks.get_mut(&key(loc))
	}
}

/// convert Vector3 to i32 tuple to use as a key in chunk array
#[inline]
fn key(loc: Vector3) -> Loc {
	let loc = loc.floor();
	(loc.x as i32, loc.y as i32, loc.z as i32)
}

#[inline]
fn fract(v: Vector3) -> Vector3 {
	Vector3::new(v.x.fract(), v.y.fract(), v.z.fract())
}

#[inline]
fn mincomp(v: Vector3) -> f32 {
	v.x.min(v.y.min(v.z))
}

fn calc_normal(hit_pos: Vector3) -> Vector3 {
	let pos_in_voxel = fract(fract(hit_pos)+Vector3::ONE);
	let centered = pos_in_voxel - Vector3::ONE*0.5;
	let axis = centered.abs().max_axis();
	axis.vec() * centered.sign()
}

fn step(e: f32, v: Vector3) -> Vector3 {
	Vector3::new(
		(v.x >= e) as u8 as f32, 
		(v.y >= e) as u8 as f32, 
		(v.z >= e) as u8 as f32,
	)
}
