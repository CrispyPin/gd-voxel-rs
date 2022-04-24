// use std::time::Instant;
use gdnative::prelude::*;
use gdnative::api::{ArrayMesh, Mesh};

use crate::common::*;
use crate::chunk::core::ChunkCore;
use crate::materials::VoxelMaterials;

pub const NORMALS: [Vector3; 6] = [
	ivec3(1, 0, 0), ivec3(-1, 0, 0),
	ivec3(0, 1, 0), ivec3(0, -1, 0),
	ivec3(0, 0, 1), ivec3(0, 0, -1)];

const FACE_VERTS: [[Vector3; 4]; 6] = [
	[ivec3(1, 0, 1), ivec3(1, 1, 1), ivec3(1, 1, 0), ivec3(1, 0, 0)],
	[ivec3(0, 0, 0), ivec3(0, 1, 0), ivec3(0, 1, 1), ivec3(0, 0, 1)],
	[ivec3(0, 1, 0), ivec3(1, 1, 0), ivec3(1, 1, 1), ivec3(0, 1, 1)],
	[ivec3(0, 0, 1), ivec3(1, 0, 1), ivec3(1, 0, 0), ivec3(0, 0, 0)],
	[ivec3(0, 0, 1), ivec3(0, 1, 1), ivec3(1, 1, 1), ivec3(1, 0, 1)],
	[ivec3(1, 0, 0), ivec3(1, 1, 0), ivec3(0, 1, 0), ivec3(0, 0, 0)]];

const QUAD_OFFSETS: [usize; 6] = [0, 1, 2, 2, 3, 0];


pub struct ChunkMesh {
	surfaces: Vec<Surface>,
	array_mesh: Ref<ArrayMesh, Shared>,
}

struct Surface {
	voxel_type: Voxel,
	vertexes: PoolArray<Vector3>,
	normals: PoolArray<Vector3>,
	// uvs: PoolArray<Vector2>,
	indexes: Int32Array,
	quad_count: usize,
	quad_capacity: usize,
}


impl ChunkMesh {
	pub fn new() -> Self {
		Self {
			surfaces: Vec::new(),//vec![Surface::new(1)],
			array_mesh: ArrayMesh::new().into_shared(),
		}
	}

	pub fn array_mesh(&self) -> &Ref<ArrayMesh, Shared> {
		&self.array_mesh
	}
	
	/// fast but suboptimal mesh
	pub fn generate_full(&mut self, core: &ChunkCore, materials: &VoxelMaterials) {
		// let start_time = Instant::now();
		for s in self.surfaces.iter_mut() {
			s.clear();
		}
		for v_index in 0..VOLUME {
			let voxel = core.voxels[v_index];
			if voxel != EMPTY {
				let surf_i = self.ensure_surface(voxel);
				self.surfaces[surf_i].allocate_batch(6, 64);
				let pos = index_to_pos(v_index);
				self.add_cube(pos, surf_i, core);
			}
		}
		// let time_taken = start_time.elapsed().as_micros() as f64 / 1000.0;
		// godot_print!("simple mesh took:  {} ms", time_taken);
		self.apply(materials);
		// if self.quad_count > 0 {
		// let time_taken = start_time.elapsed().as_micros() as f64 / 1000.0;
		// godot_print!("applying took:     {} ms", time_taken);
		// }
	}

	#[inline]
	fn add_cube(&mut self, pos: Vector3, surface_index: usize, core: &ChunkCore) {
		for face in 0..6 {
			let normal = NORMALS[face];
			if core.get_voxel(pos + normal) == EMPTY {
				let mut verts = [Vector3::ZERO; 4];
				for i in 0..4 {
					verts[i] = pos + FACE_VERTS[face][i];
				}
				self.surfaces[surface_index].add_quad(verts, face);
			}
		}
	}
	
	/// ensures a surface exists for the voxel type and returns its index
	fn ensure_surface(&mut self, voxel: Voxel) -> usize {
		let index = self.get_surface_index(voxel);
		if index.is_none() {
			self.surfaces.push(Surface::new(voxel));
			return self.surfaces.len() - 1;
		}
		index.unwrap()
	}

	fn get_surface_index(&self, voxel: Voxel) -> Option<usize> {
		for (i, surface) in self.surfaces.iter().enumerate() {
			if surface.voxel_type == voxel {
				return Some(i);
			}
		}
		None
	}

	fn apply(&mut self, materials: &VoxelMaterials) {
		// remove unused
		for s in self.surfaces.iter_mut() {
			s.trim();
		}
		let array_mesh = unsafe { self.array_mesh.assume_safe() };
		
		while array_mesh.get_surface_count() > 0 {
			array_mesh.surface_remove(0);
		}

		let mut s_count = 0;
		for s in self.surfaces.iter() {
			if s.quad_count > 0 {
				let mesh_data = s.get_array();
				array_mesh.add_surface_from_arrays(Mesh::PRIMITIVE_TRIANGLES, mesh_data, VariantArray::new_shared(), 0);
				array_mesh.surface_set_material(s_count, materials.get(s.voxel_type));
				s_count += 1;
			}
		}
	}

	
}

impl Surface {
	fn new(voxel_type: Voxel) -> Self {
		Self {
			voxel_type,
			vertexes: PoolArray::new(),
			normals: PoolArray::new(),
			// uvs: PoolArray::new(),
			indexes: PoolArray::new(),
			quad_count: 0,
			quad_capacity: 0,
		}
	}

	/// add a quad from 4 verts, in the order: [0, 1, 2, 2, 3, 0]
	fn add_quad(&mut self, corners: [Vector3; 4], face: usize/* , color: Color */) {
		let mut vertex_w = self.vertexes.write();
		let mut normal_w = self.normals.write();	
		let mut index_w = self.indexes.write();
		// let mut uv_w = self.uvs.write();
	
		for v in 0..4 {
			vertex_w[self.quad_count * 4 + v] = corners[v];
			normal_w[self.quad_count * 4 + v] = NORMALS[face];
		}

		for i in 0..6 {
			index_w[self.quad_count * 6 + i] = (self.quad_count * 4 + QUAD_OFFSETS[i]) as i32;
		}

		// uv_w[self.quad_count * 4] = Vector2::new(0.0, 1.0);
		// uv_w[self.quad_count * 4+1] = Vector2::new(0.0, 0.0);
		// uv_w[self.quad_count * 4+2] = Vector2::new(1.0, 0.0);
		// uv_w[self.quad_count * 4+3] = Vector2::new(1.0, 1.0);

		self.quad_count += 1;
	}

	fn get_array(&self) -> VariantArray {
		let mesh_data = VariantArray::new_thread_local();
		mesh_data.resize(Mesh::ARRAY_MAX as i32);
		mesh_data.set(Mesh::ARRAY_VERTEX as i32, &self.vertexes);
		mesh_data.set(Mesh::ARRAY_NORMAL as i32, &self.normals);
		mesh_data.set(Mesh::ARRAY_INDEX as i32, &self.indexes);
		// mesh_data.set(Mesh::ARRAY_TEX_UV as i32, &self.uvs);
		
		unsafe { mesh_data.assume_unique().into_shared() }
	}

	/// allocate space for additional quads
	fn resize_buffers(&mut self, amount: i32) {
		let vert_count = self.vertexes.len() + amount * 4;
		let index_count = self.indexes.len() + amount * 6;
		
		self.vertexes.resize(vert_count);
		self.normals.resize(vert_count);
		self.indexes.resize(index_count);
		// self.uvs.resize(vert_count);

		self.quad_capacity = (self.quad_capacity as i32 + amount) as usize;
	}

	/// make sure at least `min` additional quads fit; if not, resize by `batch_size`
	#[inline]
	fn allocate_batch(&mut self, min: usize, batch_size: u32) {
		if self.quad_capacity < self.quad_count + min {
			self.resize_buffers(batch_size as i32);
		}
	}

	fn trim(&mut self) {
		self.resize_buffers(self.quad_count as i32 - self.quad_capacity as i32);
	}

	fn clear(&mut self) {
		self.quad_count = 0;
		self.quad_capacity = 0;
		self.vertexes.resize(0);
		self.normals.resize(0);
		self.indexes.resize(0);
		// self.uvs.resize(0);
	}
}
