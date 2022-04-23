use gdnative::prelude::*;
use gdnative::api::{MeshInstance, RandomNumberGenerator};

mod mesh;
mod core;

use crate::common::*;
use self::mesh::*;
use self::core::*;

pub type ChunkNodeType = Spatial;


#[derive(NativeClass)]
#[inherit(ChunkNodeType)]
pub struct Chunk {
	core: ChunkCore,
	mesh: ChunkMesh,
	rng: Ref<RandomNumberGenerator, Unique>,
	needs_remesh: bool,
	location: Vector3,
}


#[methods]
impl Chunk {
	pub fn new(_owner: &ChunkNodeType) -> Self {
		Self {
			core: ChunkCore::new(),
			mesh: ChunkMesh::new(),
			rng: RandomNumberGenerator::new(),
			needs_remesh: true,
			location: Vector3::ZERO,
		}
	}

	#[export]
	fn _ready(&mut self, owner: &ChunkNodeType) {
		self.location = owner.translation();
		let mesh_instance = unsafe { 
			owner.get_node_as::<MeshInstance>("ChunkMesh")
			.unwrap()
		};
		mesh_instance.set_mesh(self.mesh.array_mesh());
		self.generate();
	}

	#[export]
	fn _process(&mut self, _owner: &ChunkNodeType, _delta: f32) {
		let input = Input::godot_singleton();
		if input.is_action_just_pressed("f3", false) {
			self.mesh.generate_simple(&self.core);
		}
		if input.is_action_just_pressed("f4", false) {
			self.randomise(0.2);
		}
		if self.needs_remesh {
			self.mesh.generate_simple(&self.core);
			self.needs_remesh = false;
		}
	}

	/// cast a ray through the chunk, source and output are in world space coords
	pub fn cast_ray(&self, source: Vector3, dir: Vector3, max_len: f32) -> RayResult {
		let source = local_pos(source);
		let mut result = self.core.cast_ray(source, dir, max_len);
		result.pos += self.location;
		result
	}

	fn randomise(&mut self, amount: f64) {
		self.core.voxels = [0; VOLUME];
		for i in 0..VOLUME {
			if self.rng.randf() < amount {
				self.core.voxels[i] = 1;
			}
		}
	}

	fn generate(&mut self) {
		if self.location.y < 0.0 {
			self.core.voxels = [3; VOLUME];
		}
		else {
			self.core.voxels = [0; VOLUME];
		}
		// torus
		for i in 0..VOLUME {
			let pos = index_to_pos(i) - ivec3(1,1,1) * 16.0 + Vector3::new(0.5, 0.5, 0.5);
			if torus(8.0, 3.0, pos.x, pos.y, pos.z) {
				self.core.voxels[i] = self.rng.randi_range(1, 255) as u8;
			}
		}

		// 3d checkerboard
		/*
		for i in 0..VOLUME {
			self.core.voxels[i] = ((i % 2 
					+ (i / WIDTH % 2)
					+ (i / AREA % 2))
				 % 2) as Voxel;
		}
		// */

		fn torus(major: f32, minor: f32, x: f32, y: f32, z: f32) -> bool {
			let q = Vector2::new(Vector2::new(x, z).length() - major, y);
			q.length() - minor < 0.0
		}
	}

	#[inline]
	pub fn get_voxel(&self, pos: Vector3) -> Voxel {
		self.core.get_voxel(pos)
	}
	
	#[inline]
	pub fn set_voxel(&mut self, pos: Vector3, voxel: Voxel) {
		self.core.set_voxel(pos, voxel);
		self.needs_remesh = true;
	}
}
