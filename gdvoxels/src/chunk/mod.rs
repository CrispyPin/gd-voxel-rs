use std::time::Instant;
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
		let wpos = (wpos / WIDTH_F).floor() * WIDTH_F;
		let node = unsafe { MeshInstance::new().assume_shared() };
		Self {
			wpos,
			loc: wpos_to_loc(wpos),
			node,
			core,
			mesh: ChunkMesh::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.core.empty
	}

	pub fn mark_empty(&mut self, state: bool) {
		self.core.empty = state;
	}

	pub fn array_mesh(&self) -> &Ref<ArrayMesh, Shared> {
		self.mesh.array_mesh()
	}

	pub fn mesh_fast(&mut self, materials: &MaterialList) {
		if self.core.empty {
			return;
		}
		let start = Instant::now();
		self.mesh.mesh_fast(&self.core, materials);
		if DEBUG_MESH_TIMES {
			let t = start.elapsed().as_micros() as f64 / 1000.0;
			godot_print!("fast mesh took {}ms", t);
		}
	}

	pub fn optimise(&mut self, materials: &MaterialList) {
		if self.core.empty {
			return;
		}
		let start = Instant::now();
		self.mesh.optimise(&self.core, materials);
		if DEBUG_MESH_TIMES {
			let t = start.elapsed().as_micros() as f64 / 1000.0;
			godot_print!("optimised mesh took {}ms", t);
		}
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
