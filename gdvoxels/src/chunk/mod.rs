use gdnative::api::{ArrayMesh, MeshInstance};
use gdnative::prelude::*;

mod mesh;
pub mod core;

use crate::common::*;
use crate::materials::MaterialList;
use self::mesh::*;
use self::core::*;


pub struct Chunk {
	pub wpos: Vector3,
	pub loc: ChunkLoc,
	pub node: Ref<MeshInstance>,
	core: ChunkCore,
	mesh: ChunkMesh,
}


impl Chunk {
	pub fn new(wpos: Vector3, core: ChunkCore) -> Self {
		let node = unsafe { MeshInstance::new().assume_shared() };
		Self {
			wpos,
			loc: wpos_to_loc(wpos),
			node,
			core,
			mesh: ChunkMesh::new(),
		}
	}

	pub fn array_mesh(&self) -> &Ref<ArrayMesh, Shared> {
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
