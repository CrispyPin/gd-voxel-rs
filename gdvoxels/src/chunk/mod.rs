use gdnative::api::ArrayMesh;
use gdnative::prelude::*;

mod mesh;
mod core;

use crate::common::*;
use crate::terrain::*;
use crate::materials::VoxelMaterials;
use self::mesh::*;
use self::core::*;

pub struct Chunk {
	pub core: ChunkCore,
	mesh: ChunkMesh,
	pub position: Vector3,
}


impl Chunk {
	pub fn new(position: Vector3, terrain_gen: &TerrainGenerator) -> Self {
		Self {
			core: ChunkCore::new(position, terrain_gen),
			mesh: ChunkMesh::new(),
			position,
		}
	}

	pub fn get_mesh(&self) -> &Ref<ArrayMesh, Shared>{
		self.mesh.array_mesh()
	}

	pub fn remesh(&mut self, materials: &VoxelMaterials) {
		self.mesh.remesh_full(&self.core, materials);
	}

	pub fn remesh_pos(&mut self, materials: &VoxelMaterials, pos: Vector3, old_voxel: Voxel) {
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
