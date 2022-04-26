use gdnative::prelude::*;
use gdnative::api::OpenSimplexNoise;
use crate::common::*;

pub struct ChunkCore {
	pub voxels: [Voxel; VOLUME],
}

impl ChunkCore {
	#[inline]
	pub fn new(pos: Vector3, rng: &Ref<OpenSimplexNoise, Unique>) -> Self {
		rng.set_octaves(5);
		let mut new_core = Self {voxels: [0; VOLUME]};
		if pos.y > WIDTH_F * 3.0 {
			return new_core;
		}
		if pos.y < WIDTH_F * -3.0 {
			return Self {voxels: [1; VOLUME]};
		}

		for x in 0..WIDTH {
			for z in 0..WIDTH {
				let height = Self::heightmap(rng, pos, x, z);
				for y in 0..WIDTH {
					let pos_y = y as f32 + pos.y;
					if  pos_y < height {
						new_core.set_voxel(Vector3::new(x as f32, y as f32, z as f32), 1);
					}
					else if pos_y < height + 2.0 {
						new_core.set_voxel(Vector3::new(x as f32, y as f32, z as f32), 2);
					}
					else if pos_y < height + 3.0 {
						new_core.set_voxel(Vector3::new(x as f32, y as f32, z as f32), 3);
					}
				}
			}
		}
		new_core
	}

	fn heightmap(rng: &Ref<OpenSimplexNoise, Unique>, chunk_pos: Vector3, x: usize, z: usize) -> f32 {
		let world_xz = Vector2::new(chunk_pos.x + x as f32, chunk_pos.z + z as f32);
		rng.get_noise_2dv(world_xz) as f32 * 32.0
	}

	#[inline]
	pub fn get_voxel(&self, pos: Vector3) -> Voxel {
		if in_bounds(pos) {
			return self.get_voxel_unsafe(pos);
		}
		EMPTY
	}
	
	#[inline]
	pub fn get_voxel_unsafe(&self, pos: Vector3) -> Voxel {
		self.voxels[pos_to_index(pos)]
	}

	#[inline]
	pub fn set_voxel(&mut self, pos: Vector3, voxel: Voxel) {
		if in_bounds(pos) {
			self.set_voxel_unsafe(pos, voxel);
		}
	}
	
	#[inline]
	pub fn set_voxel_unsafe(&mut self, pos: Vector3, voxel: Voxel) {
		self.voxels[pos_to_index(pos)] = voxel;
	}
}
