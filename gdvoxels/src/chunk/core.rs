use gdnative::prelude::*;

use crate::common::*;

pub struct ChunkCore {
	pub voxels: [Voxel; VOLUME],
}

impl ChunkCore {
	pub fn new() -> Self {
		Self {
			voxels: [0; VOLUME],
		}
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
