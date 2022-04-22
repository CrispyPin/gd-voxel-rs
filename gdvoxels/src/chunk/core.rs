use gdnative::{prelude::*, core_types::Axis};

use crate::common::*;

pub struct ChunkCore {
	pub voxels: [Voxel; VOLUME],
}

impl ChunkCore {
	pub fn new() -> Self {
		Self {
			voxels: [0; VOLUME],
		}
	}

	/// cast a ray through the chunk, source and output are in local coords
	pub fn cast_ray(&self, source: Vector3, dir: Vector3, max_len: f32) -> RayResult {
		let stepped_dir = step(0.0, dir);
		let mut ray_len = 0.0;
		
		while ray_len <= max_len {
			let ray_pos = source + dir * ray_len;
			if !in_bounds(ray_pos) {
				return RayResult::miss(ray_pos, ray_len);
			}
			let voxel = self.get_voxel(ray_pos);
			if  voxel != EMPTY {
				let normal = calc_normal(ray_pos);
				return RayResult::hit(ray_pos, normal, voxel, ray_len);
			}
			// distance "forward" along each axis to the next plane intersection
			let dist_to_next_plane = stepped_dir - fract(ray_pos);
			// distance along the ray to next plane intersection for each axis
			let deltas = dist_to_next_plane / dir;
			// move the smallest of these distances, so that no voxel is skipped
			ray_len += mincomp(deltas).max(0.0001);
		}
		RayResult::miss(source + dir * max_len, max_len)
	}

	#[inline]
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

	#[inline]
	pub fn set_voxel(&mut self, pos: Vector3, voxel: Voxel) {
		if in_bounds(pos) {
			self.set_voxel_unsafe(pos, voxel);
		}
	}
	
	#[inline]
	pub fn set_voxel_unsafe(&mut self, pos: Vector3, voxel: Voxel) {
		self.voxels[pos_to_index(pos)] = voxel;
	}
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

pub trait AxisToVector3 {
	fn vec(&self) -> Vector3;
}

impl AxisToVector3 for Axis {
	fn vec(&self) -> Vector3 {
		match self {
			Axis::X => Vector3::RIGHT,
			Axis::Y => Vector3::UP,
			Axis::Z => Vector3::BACK,
		}
	}
}
