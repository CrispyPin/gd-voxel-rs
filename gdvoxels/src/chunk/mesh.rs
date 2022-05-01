use gdnative::prelude::*;
use gdnative::api::{ArrayMesh, Mesh};

use crate::common::*;
use crate::chunk::core::ChunkCore;
use crate::materials::MaterialList;

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
	fast: Mesher,
	greedy: Mesher,
	array_mesh: Ref<ArrayMesh, Shared>,
}

struct Mesher {
	inited: bool,
	surfaces: Vec<Surface>,
	surface_types: Vec<Voxel>,
}

struct Surface {
	voxel_type: Voxel,
	vertexes: PoolArray<Vector3>,
	uvs: PoolArray<Vector2>,
	quad_count: usize,
	quad_capacity: usize,
}


impl ChunkMesh {
	pub fn new() -> Self {
		Self {
			fast: Mesher::new(),
			greedy: Mesher::new(),
			array_mesh: ArrayMesh::new().into_shared(),
		}
	}

	pub fn array_mesh(&self) -> &Ref<ArrayMesh, Shared> {
		&self.array_mesh
	}

	pub fn mesh_fast(&mut self, core: &ChunkCore, materials: &MaterialList) {
		self.fast.generate_fast(core);
		self.apply(materials, false);
	}

	pub fn optimise(&mut self, core: &ChunkCore, materials: &MaterialList) {
		self.greedy.generate_greedy(core);
		self.apply(materials, true);
	}
	
	pub fn remesh_partial(&mut self, core: &ChunkCore, materials: &MaterialList, pos: Vector3, old_voxel: Voxel) {
		if !self.fast.inited {
			self.fast.generate_fast(core)
		}
		else {
			self.fast.remesh_partial(core, pos, old_voxel);
		}
		self.apply(materials, false);
	}

	fn apply(&mut self, materials: &MaterialList, greedy: bool) {
		let array_mesh = unsafe { self.array_mesh.assume_safe() };
		array_mesh.clear_surfaces();
		if greedy {
			self.greedy.apply(&self.array_mesh, materials);
		}
		else {
			self.fast.apply(&self.array_mesh, materials);
		}
	}
}


impl Mesher {
	fn new() -> Self {
		Self {
			inited: false,
			surfaces: Vec::new(),
			surface_types: Vec::new(),
		}
	}

	fn apply(&mut self, array_mesh: &Ref<ArrayMesh>, materials: &MaterialList) {
		let array_mesh = unsafe { array_mesh.assume_safe() };

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

	/// fast but suboptimal mesh
	#[allow(unused)]
	pub fn generate_fast(&mut self, core: &ChunkCore) {
		for s in self.surfaces.iter_mut() {
			s.clear();
		}
		for v_index in 0..VOLUME {
			let voxel = core.voxels[v_index];
			if voxel != EMPTY {
				let surf_i = self.ensure_surface(voxel);
				self.surfaces[surf_i].allocate_batch(6, 64);
				let pos = index_to_vposv(v_index);
				self.add_cube(pos, surf_i, core);
			}
		}
		self.trim();
	}

	pub fn generate_greedy(&mut self, core: &ChunkCore) {
		self.inited = true;
		for s in self.surfaces.iter_mut() {
			s.clear();
		}
		for face in 0..6 {
			for layer in 0..WIDTH {
				let mut quad_strips = Vec::new();

				for slice in 0..WIDTH {
					let mut strips_active: Vec<QuadStrip> = Vec::new();
					let mut prev_top = 0;
					let mut prev = 0;
					let mut hidden_start = 0;
					
					for offset in 0..(WIDTH+1) {
						let voxel = core.get_voxel(layered_pos(face, layer, slice, offset));
						let top = core.get_voxel(layered_pos(face, layer, slice, offset) + NORMALS[face]);

						if top.is_transparent() && prev_top.is_transparent() { // remain visible
							if voxel != prev {
								if !strips_active.is_empty() {
									let mut q = strips_active.pop().unwrap();
									q.end_max = offset;
									q.end_min = offset;
									quad_strips.push(q);
								}
								if voxel.is_surface() {
									let new_quad = QuadStrip {
										voxel,
										start_min: offset,
										start_max: offset,
										end_min: offset+1,
										end_max: WIDTH,
										slice_start: slice,
										slice_end: slice+1,
										visible: true,
									};
									strips_active.push(new_quad);
								}
							}
						}
						else if top.is_opaque() && prev_top.is_transparent() { // enter under blocks
							if !strips_active.is_empty() {
								strips_active[0].end_min = offset;
							}
							hidden_start = offset;
							if voxel != prev {
								let new_quad = QuadStrip {
									voxel,
									start_min: offset,
									start_max: offset,
									end_min: offset+1,
									end_max: WIDTH,
									slice_start: slice,
									slice_end: slice+1,
									visible: false,
								};
								strips_active.push(new_quad);
							}
						}
						else if top.is_transparent() && prev_top.is_opaque() { // emerge from under blocks
							let mut i = 0;
							while i < strips_active.len() {
								if strips_active[i].voxel == voxel {
									strips_active[i].visible = true;
									i += 1;
								}
								else {
									let mut q = strips_active.remove(i);
									q.end_max = offset;
									quad_strips.push(q);
								}
							}
							if strips_active.is_empty() && voxel.is_surface() {
								let new_quad = QuadStrip {
									voxel,
									start_min: hidden_start,
									start_max: offset,
									end_min: offset+1,
									end_max: WIDTH,
									slice_start: slice,
									slice_end: slice+1,
									visible: true,
								};
								strips_active.push(new_quad);
							}
						}
						else { // remain hidden
							if voxel != prev {
								if !strips_active.iter().any(|q| q.voxel == voxel) {
									let new_quad = QuadStrip {
										voxel,
										start_min: hidden_start,
										start_max: offset,
										end_min: offset+1,
										end_max: WIDTH,
										slice_start: slice,
										slice_end: slice+1,
										visible: false,
									};
									strips_active.push(new_quad);
								}
							}
						}
						prev_top = top;
						prev = voxel;
					}
				}
				if quad_strips.is_empty() {
					continue;
				}

				// merge quads
				let mut a = 0;
				let mut b = 0;
				while a < quad_strips.len() - 1 {
					let mut main = quad_strips[a].clone();
					let other = quad_strips[b].clone();
					if main.slice_end == other.slice_start && other.voxel == main.voxel && main.visible &&
					((main.start_min >= other.start_min && main.start_min <= other.start_max) // a starts in b's range
					|| (other.start_min >= main.start_min && other.start_min <= main.start_max)) // b starts in a's range
					&&
						((main.end_min >= other.end_min && main.end_min <= other.end_max) // a ends in b's range
					|| (other.end_min >= main.end_min && other.end_min <= main.end_max)) // b ends in a's range
					 {// merge strips
						// new width
						main.slice_end = other.slice_end;
						// new valid end range
						if other.end_min > main.end_min {
							main.end_min = other.end_min;
						}
						if other.end_max < main.end_max {
							main.end_max = other.end_max;
						}
						// new valid start range
						if other.start_min > main.start_min {
							main.start_min = other.start_min;
						}
						if other.start_max < main.start_max {
							main.start_max = other.start_max;
						}
						quad_strips.remove(b);
						quad_strips[a] = main;
					}
					else {
						b += 1;
						// there is a gap between a and b in the slice width direction
						if main.slice_end < other.slice_start {
							// move forward
							a += 1;
							b = a;
						}
					}
					// b is last quad
					if b >= quad_strips.len() {
						// move forward
						a += 1;
						b = a;
					}
				}
				 
				for q in quad_strips {
					if !q.visible || q.voxel == EMPTY {
						continue;
					}
					let i = self.ensure_surface(q.voxel);
					self.surfaces[i].allocate_batch(1, 16);
					self.surfaces[i].add_quad(q.transformed_verts(face, layer), face);
				}
			}
		}
		self.trim();
	}

	fn remesh_partial(&mut self, core: &ChunkCore, pos: Vector3, old_voxel: Voxel) {
		self.inited = true;
		let voxel = core.get_voxel_unsafe(pos);

		let mut adjacent_voxels = Vec::new();
		let mut affected_surfaces = vec![self.ensure_surface(voxel), self.ensure_surface(old_voxel)];

		for normal in NORMALS {
			let v = core.get_voxel(pos - normal);
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
		self.trim();
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

	fn trim(&mut self) {
		for s in self.surfaces.iter_mut() {
			s.trim();
		}
	}
}

impl Surface {
	fn new(voxel_type: Voxel) -> Self {
		Self {
			voxel_type,
			vertexes: PoolArray::new(),
			uvs: PoolArray::new(),
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

		if DEBUG_UVS {
			let mut uv_w = self.uvs.write();
			uv_w[self.quad_count * 6] = Vector2::new(0.0, 1.0);
			uv_w[self.quad_count * 6 + 1] = Vector2::new(0.0, 0.0);
			uv_w[self.quad_count * 6 + 2] = Vector2::new(1.0, 0.0);
			uv_w[self.quad_count * 6 + 3] = Vector2::new(1.0, 0.0);
			uv_w[self.quad_count * 6 + 4] = Vector2::new(1.0, 1.0);
			uv_w[self.quad_count * 6 + 5] = Vector2::new(0.0, 1.0);
		}
		self.quad_count += 1;
	}

	fn get_array(&self) -> VariantArray {
		let mesh_data = VariantArray::new_thread_local();
		mesh_data.resize(Mesh::ARRAY_MAX as i32);
		mesh_data.set(Mesh::ARRAY_VERTEX as i32, &self.vertexes);
		if DEBUG_UVS {
			mesh_data.set(Mesh::ARRAY_TEX_UV as i32, &self.uvs);
		}
		unsafe { mesh_data.assume_unique().into_shared() }
	}

	/// allocate space for additional quads
	fn resize_buffers(&mut self, amount: i32) {
		let vert_count = self.vertexes.len() + amount * 6;
		self.vertexes.resize(vert_count);
		if DEBUG_UVS {
			self.uvs.resize(vert_count);
		}
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
		if DEBUG_UVS {
			self.uvs.resize(0);
		}
	}
}


#[derive(Clone, Debug)]
struct QuadStrip {
	voxel: Voxel,
	start_min: usize,
	start_max: usize,
	end_min: usize,
	end_max: usize,
	slice_start: usize,
	slice_end: usize,
	visible: bool,
}

impl QuadStrip {
	fn transformed_verts(&self, face: usize, layer: usize) -> [Vector3; 4] {
		match face {
			0 => [
				uvec3(layer+1, self.slice_start, self.start_max),
				uvec3(layer+1, self.slice_start, self.end_min),
				uvec3(layer+1, self.slice_end, self.end_min),
				uvec3(layer+1, self.slice_end, self.start_max),
			],
			1 => [
				uvec3(layer, self.slice_end, self.start_max),
				uvec3(layer, self.slice_end, self.end_min),
				uvec3(layer, self.slice_start, self.end_min),
				uvec3(layer, self.slice_start, self.start_max),
			],
			2 => [
				uvec3(self.start_max, layer+1, self.slice_start),
				uvec3(self.end_min, layer+1, self.slice_start),
				uvec3(self.end_min, layer+1, self.slice_end),
				uvec3(self.start_max, layer+1, self.slice_end),
			],
			3 => [
				uvec3(self.start_max, layer, self.slice_end),
				uvec3(self.end_min, layer, self.slice_end),
				uvec3(self.end_min, layer, self.slice_start),
				uvec3(self.start_max, layer, self.slice_start),
			],
			4 => [
				uvec3(self.slice_start, self.start_max, layer+1),
				uvec3(self.slice_start, self.end_min, layer+1),
				uvec3(self.slice_end, self.end_min, layer+1),
				uvec3(self.slice_end, self.start_max, layer+1),
			],
			5 => [
				uvec3(self.slice_end, self.start_max, layer),
				uvec3(self.slice_end, self.end_min, layer),
				uvec3(self.slice_start, self.end_min, layer),
				uvec3(self.slice_start, self.start_max, layer),
			],
			_ => panic!("invalid face index in QuadStrip.transformed_verts")
		}
	}
}


fn layered_pos(face: usize, layer: usize, slice: usize, offset: usize) -> Vector3 {
	match face {
		0 | 1 => uvec3(layer, slice, offset),
		2 | 3 => uvec3(offset, layer, slice),
		4 | 5 => uvec3(slice, offset, layer),
		_ => panic!("invalid face index for layered_pos()")
	}
}
