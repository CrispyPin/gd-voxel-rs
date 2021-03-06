use gdnative::prelude::*;

use crate::common::*;


pub struct ChunkCore {
	pub empty: bool,
	pub voxels: Box<[Voxel; VOLUME]>,
}


impl ChunkCore {
	pub fn new() -> Self {
		Self {
			empty: true,
			// create array on the heap
			voxels: vec![0u8; VOLUME].into_boxed_slice().try_into().unwrap()
		}
	}

	#[allow(unused)]
	pub fn new_filled(v: Voxel) -> Self {
		Self {
			empty: v == EMPTY,
			voxels: vec![v; VOLUME].into_boxed_slice().try_into().unwrap()
		}
	}

	#[inline]
	pub fn get_voxel(&self, vposv: Vector3) -> Voxel {
		if vposv_in_bounds(vposv) {
			return self.get_voxel_unsafe(vposv);
		}
		EMPTY
	}

	#[inline]
	pub fn get_voxel_i(&self, vpos: VoxelPos) -> Voxel {
		if vpos_in_bounds(vpos) {
			return self.voxels[vpos_to_index(vpos)];
		}
		EMPTY
	}
	
	#[inline]
	pub fn get_voxel_unsafe(&self, pos: Vector3) -> Voxel {
		self.voxels[vposv_to_index(pos)]
	}

	#[inline]
	pub fn set_voxel(&mut self, pos: Vector3, voxel: Voxel) {
		if vposv_in_bounds(pos) {
			self.set_voxel_unsafe(pos, voxel);
		}
	}
	
	#[inline]
	pub fn set_voxel_unsafe(&mut self, pos: Vector3, voxel: Voxel) {
		self.voxels[vposv_to_index(pos)] = voxel;
	}
}
