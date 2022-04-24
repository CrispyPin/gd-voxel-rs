use gdnative::{prelude::*, core_types::Axis};

pub const WIDTH: usize = 32;
pub const AREA: usize = WIDTH * WIDTH;
pub const VOLUME: usize = AREA * WIDTH;

pub const WIDTH_F: f32 = WIDTH as f32;

pub type Voxel = u8;
pub const EMPTY: Voxel = 0;

/// convert world coordinate to a chunk location
pub fn chunk_loc(world_pos: Vector3) -> Vector3 {
	(world_pos / WIDTH_F).floor()
}

/// convert world coordinate to a position within the chunk
pub fn local_pos(world_pos: Vector3) -> Vector3 {
	world_pos.floor().posmod(WIDTH_F)
}

/// [i32] to [Vector3]
pub const fn ivec3(x: i32, y: i32, z: i32) -> Vector3 {
	Vector3::new(x as f32, y as f32, z as f32)
}

#[inline]
pub fn in_bounds(pos: Vector3) -> bool{
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

pub trait VoxelData {
	fn name(&self) -> String;
}

impl VoxelData for Voxel {
	fn name(&self) -> String {
		match *self {
			0 => "air".into(),
			1 => "stone".into(),
			2 => "dirt".into(),
			3 => "grass".into(),
			other => format!("{}", other),
		}
	}
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
