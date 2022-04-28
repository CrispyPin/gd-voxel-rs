use gdnative::api::ArrayMesh;
use gdnative::prelude::*;

mod mesh;
pub mod core;

use crate::common::*;
use crate::materials::MaterialList;
use self::mesh::*;
use self::core::*;

pub struct Chunk {
	core: ChunkCore,
	mesh: ChunkMesh,
	pub position: Vector3,
}


impl Chunk {
	pub fn new(position: Vector3, core: ChunkCore) -> Self {
		Self {
			core,
			mesh: ChunkMesh::new(),
			position,
		}
	}

	pub fn get_mesh(&self) -> &Ref<ArrayMesh, Shared>{
		self.mesh.array_mesh()
	}

	pub fn remesh(&mut self, materials: &MaterialList) {
		self.mesh.remesh_full(&self.core, materials);
	}

	pub fn remesh_pos(&mut self, materials: &MaterialList, pos: Vector3, old_voxel: Voxel) {
		self.mesh.remesh_partial(&self.core, materials, pos, old_voxel);
	}

	#[inline]
	pub fn get_voxel(&self, pos: Vector3) -> Voxel {
		self.core.get_voxel(pos)
	}
	
	#[inline]
	pub fn set_voxel(&mut self, pos: Vector3, voxel: Voxel) {
		self.core.set_voxel(pos, voxel);
	}
}
