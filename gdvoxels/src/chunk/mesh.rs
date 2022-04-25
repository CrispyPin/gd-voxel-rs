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
	surface_types: Vec<Voxel>,
	array_mesh: Ref<ArrayMesh, Shared>,
}

struct Surface {
	voxel_type: Voxel,
	vertexes: PoolArray<Vector3>,
	quad_count: usize,
	quad_capacity: usize,
}


impl ChunkMesh {
	pub fn new() -> Self {
		Self {
			surfaces: Vec::new(),
			surface_types: Vec::new(),
			array_mesh: ArrayMesh::new().into_shared(),
		}
	}

	pub fn array_mesh(&self) -> &Ref<ArrayMesh, Shared> {
		&self.array_mesh
	}
	
	/// fast but suboptimal mesh
	pub fn remesh_full(&mut self, core: &ChunkCore, materials: &VoxelMaterials) {
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
		self.apply(materials);
	}

	pub fn remesh_partial(&mut self, core: &ChunkCore, materials: &VoxelMaterials, pos: Vector3, old_voxel: Voxel) {
		let voxel = core.get_voxel_unsafe(pos);

		let mut adjacent_voxels = Vec::new();
		let mut affected_surfaces = vec![self.ensure_surface(voxel), self.ensure_surface(old_voxel)];

		for face in 0..6 {
			let v = core.get_voxel(pos - NORMALS[face]);
			adjacent_voxels.push(v);
			affected_surfaces.push(self.ensure_surface(v));
		}

		// remove affected quads
		for surf_i in affected_surfaces.iter().filter(|&i| *i != usize::MAX) {
			self.surfaces[*surf_i].remove_quads_in_bound(pos - Vector3::ONE*0.1, pos + Vector3::ONE*1.1);
		}

		if voxel != EMPTY {
			let surf_i = affected_surfaces[0];
			self.surfaces[surf_i].allocate_batch(6, 6);
			self.add_cube(pos, surf_i, core);
		}
		else { // set faces for surrounding voxels; essentially an inverted version of the other case
			for face in 0..6 {
				let other_voxel = adjacent_voxels[face];
				if other_voxel != EMPTY {
					let other_pos = pos - NORMALS[face];
					let surf_i = affected_surfaces[face + 2];
					self.surfaces[surf_i].allocate_batch(6, 6);

					let verts = [
						other_pos + FACE_VERTS[face][0],
						other_pos + FACE_VERTS[face][1],
						other_pos + FACE_VERTS[face][2],
						other_pos + FACE_VERTS[face][3],
					];
					self.surfaces[surf_i].add_quad(verts, face);
				}
			}
		}
		self.apply(materials);
	}

	#[inline]
	fn add_cube(&mut self, pos: Vector3, surface_index: usize, core: &ChunkCore) {
		for face in 0..6 {
			let normal = NORMALS[face];
			if core.get_voxel(pos + normal) == EMPTY {
				let verts = [
					pos + FACE_VERTS[face][0],
					pos + FACE_VERTS[face][1],
					pos + FACE_VERTS[face][2],
					pos + FACE_VERTS[face][3],
				];
				self.surfaces[surface_index].add_quad(verts, face);
			}
		}
	}
	
	/// ensures a surface exists for the voxel type and returns its index
	/// if the requested voxel type is air, usize::MAX is returned instead
	#[inline]
	fn ensure_surface(&mut self, voxel: Voxel) -> usize {
		if voxel == EMPTY {
			return usize::MAX;
		}
		let index = self.get_surface_index(voxel);
		if index.is_none() {
			self.surfaces.push(Surface::new(voxel));
			self.surface_types.push(voxel);
			return self.surfaces.len() - 1;
		}
		index.unwrap()
	}

	#[inline]
	fn get_surface_index(&self, voxel: Voxel) -> Option<usize> {
		for (i, v) in self.surface_types.iter().enumerate() {
			if *v == voxel {
				return Some(i);
			}
		}
		None
	}

	fn apply(&mut self, materials: &VoxelMaterials) {
		for s in self.surfaces.iter_mut() {
			s.trim();
		}
		let array_mesh = unsafe { self.array_mesh.assume_safe() };
		array_mesh.clear_surfaces();

		let mut surf_i = 0;
		while surf_i < self.surfaces.len() {
			let s = &self.surfaces[surf_i];
			if s.quad_count > 0 {
				let mesh_data = s.get_array();
				array_mesh.add_surface_from_arrays(Mesh::PRIMITIVE_TRIANGLES, mesh_data, VariantArray::new_shared(), 0);
				array_mesh.surface_set_material(surf_i as i64, materials.get(s.voxel_type));
				surf_i += 1;
			}
			else {
				self.surfaces.remove(surf_i);
				self.surface_types.remove(surf_i);
			}
		}
	}
}

impl Surface {
	fn new(voxel_type: Voxel) -> Self {
		Self {
			voxel_type,
			vertexes: PoolArray::new(),
			quad_count: 0,
			quad_capacity: 0,
		}
	}

	fn remove_quads_in_bound(&mut self, pos_min: Vector3, pos_max: Vector3) {
		let mut quad_i = 0;
		while quad_i < self.quad_count {
			if (0..6).all(|i| 
				in_box(self.vertexes.get(quad_i as i32 * 6 + i), pos_min, pos_max))
				{
				self.remove_quad(quad_i);
			}
			else {
				quad_i += 1;
			}
		}

		#[inline]
		fn in_box(v: Vector3, min: Vector3, max: Vector3) -> bool {
			v.x <= max.x && v.x >= min.x
			&& v.y <= max.y && v.y >= min.y
			&& v.z <= max.z && v.z >= min.z
		}
	}

	/// removes a quad by moving a quad from the end of the list to its location
	/// would probably crash if only one exists, but the lowest it can be is 6 so it shouldTM not happen
	fn remove_quad(&mut self, to_remove: usize) {
		self.quad_count -= 1;
		let replacement = self.quad_count;
		if to_remove == replacement {
			return;
		}
		let mut vertex_w = self.vertexes.write();
		for v in 0..6 {
			vertex_w[to_remove * 6 + v] = vertex_w[replacement * 6 + v];
		}
	}

	/// add a quad from 4 verts, in the order: [0, 1, 2, 2, 3, 0]
	#[inline]
	fn add_quad(&mut self, corners: [Vector3; 4], face: usize) {
		let mut vertex_w = self.vertexes.write();
		let encoded_normal = Vector3::new(face as f32 / 100.0 + 0.005, 0.0, 0.0);
		for v in 0..6 {
			vertex_w[self.quad_count * 6 + v] = corners[QUAD_OFFSETS[v]] + encoded_normal;
		}
		self.quad_count += 1;
	}

	fn get_array(&self) -> VariantArray {
		let mesh_data = VariantArray::new_thread_local();
		mesh_data.resize(Mesh::ARRAY_MAX as i32);
		mesh_data.set(Mesh::ARRAY_VERTEX as i32, &self.vertexes);
		unsafe { mesh_data.assume_unique().into_shared() }
	}

	/// allocate space for additional quads
	fn resize_buffers(&mut self, amount: i32) {
		let vert_count = self.vertexes.len() + amount * 6;
		self.vertexes.resize(vert_count);
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
	}
}
