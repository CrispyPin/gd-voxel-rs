use gdnative::prelude::*;
use gdnative::api::{ArrayMesh, MeshInstance, RandomNumberGenerator, Mesh};

use crate::common::*;

const NORMALS: [Vector3; 6] = [
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

pub type ChunkNodeType = Spatial;

#[derive(NativeClass)]
#[inherit(ChunkNodeType)]
pub struct Chunk {
	voxels: [Voxel; VOLUME],
	array_mesh: Ref<ArrayMesh, Shared>,
	mesh_vertex: PoolArray<Vector3>,
	mesh_normal: PoolArray<Vector3>,
	mesh_uv: PoolArray<Vector2>,
	mesh_index: Int32Array,
	mesh_index_offset: usize,
	rng: Ref<RandomNumberGenerator, Unique>,
	needs_remesh: bool,
	location: Vector3,
}


#[methods]
impl Chunk {
	pub fn new(_owner: &ChunkNodeType) -> Self {
		Self {
			voxels: [0; VOLUME],
			array_mesh: ArrayMesh::new().into_shared(),
			mesh_vertex: PoolArray::new(),
			mesh_normal: PoolArray::new(),
			mesh_uv: PoolArray::new(),
			mesh_index: PoolArray::new(),
			mesh_index_offset: 0,
			rng: RandomNumberGenerator::new(),
			needs_remesh: true,
			location: Vector3::ZERO,
		}
	}

	#[export]
	fn _ready(&mut self, owner: &ChunkNodeType) {
		self.location = owner.translation();
		let mesh_instance = unsafe { 
			owner.get_node_as::<MeshInstance>("ChunkMesh")
			.unwrap()
		};
		mesh_instance.set_mesh(&self.array_mesh);
		self.generate();
	}

	#[export]
	fn _process(&mut self, _owner: &ChunkNodeType, _delta: f32) {
		let input = Input::godot_singleton();
		if input.is_action_just_pressed("f3", false) {
			self.mesh_simple();
		}
		if input.is_action_just_pressed("f4", false) {
			self.randomise(0.2);
		}
		if self.needs_remesh {
			self.mesh_simple();
			self.needs_remesh = false;
		}
	}

	/// cast a ray through the chunk, source and output are in world space coords
	pub fn cast_ray(&self, source: Vector3, dir: Vector3, max_len: f32) -> RayResult {
		let source = local_pos(source);

		let mut ray_len = 0.0;

		// 
		let stepped_dir = step(0.0, dir);
		
		while ray_len <= max_len {
			let ray_pos = source + dir * ray_len;
			if !in_bounds(ray_pos) {
				return RayResult::miss(ray_pos + self.location, ray_len);
			}
			let voxel = self.get_voxel(ray_pos);
			if  voxel != EMPTY {
				let normal = calc_normal(ray_pos);
				return RayResult::hit(ray_pos + self.location, normal, voxel, ray_len);
			}

			// distance "forward" along each axis to the next plane intersection
			let dist_to_next_plane = stepped_dir - fract(ray_pos);
			// distance along the ray to next plane intersection for each axis
			let deltas = dist_to_next_plane / dir;
			// move the smallest of these distances, so that no voxel is skipped
			ray_len += mincomp(deltas).max(0.0001);
		}
		RayResult::miss(source + dir * max_len + self.location, max_len)
	}

	/// fast but suboptimal mesh
	fn mesh_simple(&mut self) {
		self.clear_mesh_data();
		let mut quad_capacity = 0;

		for index in 0..VOLUME {
			if self.voxels[index] != EMPTY {
				if self.mesh_index_offset + 6 > quad_capacity {
					self.resize_mesh_data(64);
					quad_capacity += 64;
				}
				let pos = index_to_pos(index);
				for face in 0..6 {
					let normal = NORMALS[face];
					if self.get_voxel(pos + normal) != EMPTY {
						continue;
					}
					let mut verts = [Vector3::ZERO; 4];
					for i in 0..4 {
						verts[i] = pos + FACE_VERTS[face][i];
					}
					self.mesh_quad(verts, face);
				}
			}
		}
		self.resize_mesh_data(self.mesh_index_offset as i32 - quad_capacity as i32);
		self.apply_mesh_data();
	}

	/// add a quad from 4 verts, in the order: [0, 1, 2, 2, 3, 0]
	fn mesh_quad(&mut self, verts: [Vector3; 4], face: usize) {
		let mut vertex_w = self.mesh_vertex.write();
		let mut normal_w = self.mesh_normal.write();	
		let mut index_w = self.mesh_index.write();
	
		// let color_w = self.mesh_color.write();
		// let col = Color::from_rgb(rng.randf(), rng.randf(), rng.randf());
	
		for v in 0..4 {
			vertex_w[self.mesh_index_offset * 4 + v] = verts[v];
			normal_w[self.mesh_index_offset * 4 + v] = NORMALS[face];
			// color_w[mesh_index_offset * 4 + v] = col;
		}

		for i in 0..6 {
			index_w[self.mesh_index_offset * 6 + i] = (self.mesh_index_offset * 4 + QUAD_OFFSETS[i]) as i32;
		}

		let mut uv_w = self.mesh_uv.write();
		uv_w[self.mesh_index_offset * 4] = Vector2::new(0.0, 1.0);
		uv_w[self.mesh_index_offset * 4+1] = Vector2::new(0.0, 0.0);
		uv_w[self.mesh_index_offset * 4+2] = Vector2::new(1.0, 0.0);
		uv_w[self.mesh_index_offset * 4+3] = Vector2::new(1.0, 1.0);

		self.mesh_index_offset += 1;
	}

	/// allocate more space for `quad_count` more quads
	fn resize_mesh_data(&mut self, quad_count: i32) {
		let vert_count = self.mesh_vertex.len() + quad_count * 4;
		let index_count = self.mesh_index.len() + quad_count * 6;
		
		self.mesh_vertex.resize(vert_count);
		self.mesh_normal.resize(vert_count);
		self.mesh_index.resize(index_count);

		// mesh_color.resize(vert_count);
		self.mesh_uv.resize(vert_count);
	}

	fn apply_mesh_data(&mut self) {
		let mesh_data = VariantArray::new_thread_local();
		mesh_data.resize(Mesh::ARRAY_MAX as i32);
		mesh_data.set(Mesh::ARRAY_VERTEX as i32, &self.mesh_vertex);
		mesh_data.set(Mesh::ARRAY_NORMAL as i32, &self.mesh_normal);
		mesh_data.set(Mesh::ARRAY_INDEX as i32, &self.mesh_index);
		mesh_data.set(Mesh::ARRAY_TEX_UV as i32, &self.mesh_uv);
		
		let mesh_data = unsafe { mesh_data.assume_unique().into_shared() };
		let array_mesh = unsafe { self.array_mesh.assume_safe() };

		if array_mesh.get_surface_count() > 0 {
			array_mesh.surface_remove(0);
		}

		array_mesh.add_surface_from_arrays(Mesh::PRIMITIVE_TRIANGLES, mesh_data, VariantArray::new_shared(), 0);
	}

	fn clear_mesh_data(&mut self) {
		self.mesh_index_offset = 0;
		self.mesh_vertex.resize(0);
		self.mesh_normal.resize(0);
		self.mesh_index.resize(0);
		self.mesh_uv.resize(0);
	}

	fn randomise(&mut self, amount: f64) {
		self.voxels = [0; VOLUME];
		for i in 0..VOLUME {
			if self.rng.randf() < amount {
				self.voxels[i] = 1;
			}
		}
	}

	fn generate(&mut self) {
		self.voxels = [0; VOLUME];
		// grid
		for i in 0..WIDTH {
			self.set_voxel_unsafe(uvec3(i, 0, 0), 1);
			self.set_voxel_unsafe(uvec3(i, 0, WIDTH-1), 1);
			self.set_voxel_unsafe(uvec3(i, WIDTH-1, 0), 1);
			self.set_voxel_unsafe(uvec3(i, WIDTH-1, WIDTH-1), 1);
			
			self.set_voxel_unsafe(uvec3(0, 0, i), 1);
			self.set_voxel_unsafe(uvec3(0, WIDTH-1, i), 1);
			self.set_voxel_unsafe(uvec3(WIDTH-1, 0, i), 1);
			self.set_voxel_unsafe(uvec3(WIDTH-1, WIDTH-1, i), 1);

			self.set_voxel_unsafe(uvec3(0, i, 0), 1);
			self.set_voxel_unsafe(uvec3(0, i, WIDTH-1), 1);
			self.set_voxel_unsafe(uvec3(WIDTH-1, i, 0), 1);
			self.set_voxel_unsafe(uvec3(WIDTH-1, i, WIDTH-1), 1);
		}

		// 3d checkerboard
		/*
		for i in 0..VOLUME {
			self.voxels[i] = ((i % 2 
					+ (i / WIDTH % 2)
					+ (i / AREA % 2))
				 % 2) as Voxel;
		}
		*/

		// torus
		for i in 0..VOLUME {
			let pos = index_to_pos(i) - ivec3(1,1,1) * 16.0 + Vector3::new(0.5, 0.5, 0.5);
			if torus(10.0, 5.0, pos.x, pos.y, pos.z) {
				self.voxels[i] = 1;
			}
		}

		fn torus(major: f32, minor: f32, x: f32, y: f32, z: f32) -> bool {
			let q = Vector2::new(Vector2::new(x, z).length() - major, y);
			q.length() - minor < 0.0
		}
	}

	pub fn get_voxel(&self, pos: Vector3) -> Voxel {
		if in_bounds(pos) {
			return self.get_voxel_unsafe(pos);
		}
		EMPTY
	}
	
	#[inline]
	pub fn get_voxel_unsafe(&self, pos: Vector3) -> Voxel {
		self.voxels[pos_to_index(pos)]
	}

	pub fn set_voxel(&mut self, pos: Vector3, voxel: Voxel) {
		if in_bounds(pos) {
			self.set_voxel_unsafe(pos, voxel);
		}
	}
	
	#[inline]
	pub fn set_voxel_unsafe(&mut self, pos: Vector3, voxel: Voxel) {
		self.voxels[pos_to_index(pos)] = voxel;
		self.needs_remesh = true;
	}
}

#[inline]
fn in_bounds(pos: Vector3) -> bool{
	const WIDTH_F: f32 = WIDTH as f32;
	pos.x >= 0.0 && pos.x < WIDTH_F &&
	pos.y >= 0.0 && pos.y < WIDTH_F &&
	pos.z >= 0.0 && pos.z < WIDTH_F
}

#[inline]
fn pos_to_index(pos: Vector3) -> usize {
	pos.x as usize * AREA
	+ pos.y as usize * WIDTH
	+ pos.z as usize
}

#[inline]
fn index_to_pos(i: usize) -> Vector3 {
	Vector3::new(
		((i / AREA) as f32).floor(),
		((i/WIDTH % WIDTH) as f32).floor(),
		(i % WIDTH) as f32
	)
}

#[inline]
fn fract(v: Vector3) -> Vector3 {
	Vector3::new(v.x.fract(), v.y.fract(), v.z.fract())
}

#[inline]
fn mincomp(v: Vector3) -> f32 {
	v.x.min(v.y.min(v.z))
}

fn calc_normal(hit_pos: Vector3) -> Vector3 {
	let pos_in_voxel = fract(hit_pos);
	let centered = pos_in_voxel - Vector3::ONE*0.5;
	let axis = centered.abs().max_axis();
	axis.vec() * centered.sign()
}


fn step(e: f32, v: Vector3) -> Vector3 {
	Vector3::new(
		(v.x >= e) as u8 as f32, 
		(v.y >= e) as u8 as f32, 
		(v.z >= e) as u8 as f32,
	)
}
