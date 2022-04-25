use gdnative::api::ArrayMesh;
use gdnative::prelude::*;

mod mesh;
mod core;

use crate::common::*;
use crate::materials::VoxelMaterials;
use self::mesh::*;
use self::core::*;

pub struct Chunk {
	pub core: ChunkCore,
	mesh: ChunkMesh,
	location: Vector3,
}


impl Chunk {
	pub fn new(location: Vector3) -> Self {
		let mut instance = Self {
			core: ChunkCore::new(),
			mesh: ChunkMesh::new(),
			location,
		};
		instance.generate();
		instance
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

	pub fn generate(&mut self) {
		if self.location.y < 0.0 {
			self.core.voxels = [3; VOLUME];
		}
		else if self.location.y < 1.0 {
			// torus
			for i in 0..VOLUME {
				if index_to_pos(i).y < 8.0 {
					self.core.voxels[i] = 1;
				}
				let pos = index_to_pos(i) - ivec3(1,1,1) * 8.0 + Vector3::new(0.5, 0.5, 0.5);
				if torus(5.0, 2.0, pos.x, pos.y, pos.z) {
					self.core.voxels[i] = 2;
				}
			}
		}
		/* else if self.location.y < WIDTH_F+1.0 {
			// 3d checkerboard
			for i in 0..VOLUME {
				self.core.voxels[i] = ((i % 2 
						+ (i / WIDTH % 2)
						+ (i / AREA % 2))
					 % 2) as Voxel;
			}
		} */
		else {
			self.core.voxels = [0; VOLUME];
		}

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
	}
}
