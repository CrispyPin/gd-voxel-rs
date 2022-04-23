use gdnative::prelude::*;

pub const WIDTH: usize = 32;
pub const AREA: usize = WIDTH * WIDTH;
pub const VOLUME: usize = AREA * WIDTH;

pub type Voxel = u8;

pub const EMPTY: Voxel = 0;

/// convert world coordinate to a chunk location
pub fn chunk_loc(world_pos: Vector3) -> Vector3 {
	(world_pos / WIDTH as f32).floor()
}

/// convert world coordinate to a position within the chunk
pub fn local_pos(world_pos: Vector3) -> Vector3 {
	world_pos.posmod(WIDTH as f32)
}

/// [i32] to [Vector3]
pub const fn ivec3(x: i32, y: i32, z: i32) -> Vector3 {
	Vector3::new(x as f32, y as f32, z as f32)
}

/// [usize] to [Vector3]
pub const fn uvec3(x: usize, y: usize, z: usize) -> Vector3 {
	Vector3::new(x as f32, y as f32, z as f32)
}

#[inline]
pub fn in_bounds(pos: Vector3) -> bool{
	const WIDTH_F: f32 = WIDTH as f32;
	pos.x >= 0.0 && pos.x < WIDTH_F &&
	pos.y >= 0.0 && pos.y < WIDTH_F &&
	pos.z >= 0.0 && pos.z < WIDTH_F
}

#[inline]
pub fn pos_to_index(pos: Vector3) -> usize {
	pos.x as usize * AREA
	+ pos.y as usize * WIDTH
	+ pos.z as usize
}

#[inline]
pub fn index_to_pos(i: usize) -> Vector3 {
	Vector3::new(
		((i / AREA) as f32).floor(),
		((i/WIDTH % WIDTH) as f32).floor(),
		(i % WIDTH) as f32
	)
}

#[derive(ToVariant)]
pub struct RayResult {
	pub hit: bool,
	pub pos: Vector3,
	pub normal: Vector3,
	pub voxel: Voxel,
	pub distance: f32,
}

impl RayResult {
	pub fn hit(pos: Vector3, normal: Vector3, voxel: Voxel, distance: f32) -> Self {
		Self {
			hit: true,
			pos,
			normal,
			voxel,
			distance,
		}
	}

	pub fn miss(pos: Vector3, distance: f32) -> Self {
		Self {
			hit: false,
			pos,
			normal: Vector3::ZERO,
			voxel: EMPTY,
			distance,
		}
	}
}

pub trait VoxelColour {
	fn color(&self) -> Color;
}

impl VoxelColour for Voxel {
	fn color(&self) -> Color {
		Color::from_hsv(*self as f32 / 256.0, 0.5, 1.0)
	}
}
