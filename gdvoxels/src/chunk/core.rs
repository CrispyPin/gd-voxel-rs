use gdnative::prelude::*;
use crate::common::*;
use crate::terrain::*;


#[derive(Clone)]
pub struct ChunkCore {
	pub voxels: [Voxel; VOLUME],
}


impl ChunkCore {
	#[inline]
	pub fn new(pos: Vector3, terrain_gen: &TerrainGenerator) -> Self {
		let mut new_core = Self {voxels: [0; VOLUME]};
		if pos.y > WIDTH_F * 4.0 {
			return new_core;
		}
		if pos.y < WIDTH_F * -4.0 {
			return Self {voxels: [1; VOLUME]};
		}

		for x in 0..WIDTH {
			for z in 0..WIDTH {
				let world_x = x as f64 + pos.x as f64;
				let world_z = z as f64 + pos.z as f64;
				let height = terrain_gen.height(world_x, world_z) as f32;
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
