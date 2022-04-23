use std::time::Instant;

use gdnative::prelude::*;
use gdnative::api::{ArrayMesh, Mesh};

use crate::common::*;
use crate::chunk::core::ChunkCore;

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
	vertexes: PoolArray<Vector3>,
	normals: PoolArray<Vector3>,
	colors: PoolArray<Color>,
	uvs: PoolArray<Vector2>,
	indexes: Int32Array,
	quad_count: usize,
	quad_capacity: usize,
	array_mesh: Ref<ArrayMesh, Shared>,
}

impl ChunkMesh {
	pub fn new() -> Self {
		Self {
			vertexes: PoolArray::new(),
			normals: PoolArray::new(),
			uvs: PoolArray::new(),
			indexes: PoolArray::new(),
			colors: PoolArray::new(),
			quad_count: 0,
			quad_capacity: 0,
			array_mesh: ArrayMesh::new().into_shared(),
		}
	}

	pub fn array_mesh(&self) -> &Ref<ArrayMesh, Shared> {
		&self.array_mesh
	}
	
	/// fast but suboptimal mesh
	pub fn generate_simple(&mut self, core: &ChunkCore) {
		let start_time = Instant::now();
		self.clear();
		for index in 0..VOLUME {
			let voxel = core.voxels[index];
			if voxel != EMPTY {
				self.allocate_batch(6, 64);
				let pos = index_to_pos(index);
				self.add_cube(pos, voxel, core);
			}
		}
		self.apply();
		let time_taken = start_time.elapsed().as_micros() as f64 / 1000.0;
		// godot_print!("simple mesh took: {} ms", time_taken);
	}

	#[inline]
	fn add_cube(&mut self, pos: Vector3, voxel: Voxel, core: &ChunkCore) {
		for face in 0..6 {
			let normal = NORMALS[face];
			if core.get_voxel(pos + normal) == EMPTY {
				let mut verts = [Vector3::ZERO; 4];
				for i in 0..4 {
					verts[i] = pos + FACE_VERTS[face][i];
				}
				self.add_quad(verts, face, voxel.color());
			}
		}
	}
	
	/// add a quad from 4 verts, in the order: [0, 1, 2, 2, 3, 0]
	#[inline]
	fn add_quad(&mut self, corners: [Vector3; 4], face: usize, color: Color) {
		let mut vertex_w = self.vertexes.write();
		let mut normal_w = self.normals.write();	
		let mut index_w = self.indexes.write();
		let mut uv_w = self.uvs.write();
		let mut color_w = self.colors.write();
	
		for v in 0..4 {
			vertex_w[self.quad_count * 4 + v] = corners[v];
			normal_w[self.quad_count * 4 + v] = NORMALS[face];
			color_w[self.quad_count * 4 + v] = color;
		}

		for i in 0..6 {
			index_w[self.quad_count * 6 + i] = (self.quad_count * 4 + QUAD_OFFSETS[i]) as i32;
		}

		uv_w[self.quad_count * 4] = Vector2::new(0.0, 1.0);
		uv_w[self.quad_count * 4+1] = Vector2::new(0.0, 0.0);
		uv_w[self.quad_count * 4+2] = Vector2::new(1.0, 0.0);
		uv_w[self.quad_count * 4+3] = Vector2::new(1.0, 1.0);

		self.quad_count += 1;
	}

	/// allocate space for additional quads
	fn resize_buffers(&mut self, amount: i32) {
		let vert_count = self.vertexes.len() + amount * 4;
		let index_count = self.indexes.len() + amount * 6;
		
		self.vertexes.resize(vert_count);
		self.normals.resize(vert_count);
		self.indexes.resize(index_count);
		self.colors.resize(vert_count);
		self.uvs.resize(vert_count);

		self.quad_capacity = (self.quad_capacity as i32 + amount) as usize;
	}

	/// make sure at least `min` additional quads fit; if not, resize by `batch_size`
	#[inline]
	fn allocate_batch(&mut self, min: usize, batch_size: u32) {
		if self.quad_capacity < self.quad_count + min {
			self.resize_buffers(batch_size as i32);
		}
	}

	fn apply(&mut self) {
		// remove unused
		self.resize_buffers(self.quad_count as i32 - self.quad_capacity as i32);
		
		let mesh_data = VariantArray::new_thread_local();
		mesh_data.resize(Mesh::ARRAY_MAX as i32);
		mesh_data.set(Mesh::ARRAY_VERTEX as i32, &self.vertexes);
		mesh_data.set(Mesh::ARRAY_NORMAL as i32, &self.normals);
		mesh_data.set(Mesh::ARRAY_INDEX as i32, &self.indexes);

		mesh_data.set(Mesh::ARRAY_TEX_UV as i32, &self.uvs);
		mesh_data.set(Mesh::ARRAY_COLOR as i32, &self.colors);
		
		let mesh_data = unsafe { mesh_data.assume_unique().into_shared() };
		let array_mesh = unsafe { self.array_mesh.assume_safe() };

		if array_mesh.get_surface_count() > 0 {
			array_mesh.surface_remove(0);
		}

		if self.quad_count > 0 {
			array_mesh.add_surface_from_arrays(Mesh::PRIMITIVE_TRIANGLES, mesh_data, VariantArray::new_shared(), 0);
		}
	}

	fn clear(&mut self) {
		self.quad_count = 0;
		self.quad_capacity = 0;
		self.vertexes.resize(0);
		self.normals.resize(0);
		self.indexes.resize(0);
		self.uvs.resize(0);
		self.colors.resize(0);
	}
}