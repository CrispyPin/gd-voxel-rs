use gdnative::prelude::*;
use gdnative::core_types::Axis;

pub const WIDTH: usize = 32;
pub const AREA: usize = WIDTH * WIDTH;
pub const VOLUME: usize = AREA * WIDTH;

pub const WIDTH_F: f32 = WIDTH as f32;
pub const WIDTH_I8: i8 = WIDTH as i8;

pub type Voxel = u8;
pub const EMPTY: Voxel = 0;

pub const DEBUG_UVS: bool = true;
pub const DEBUG_MESH_TIMES: bool = false;

/// Represents a chunk location
/// Loc(1,2,3) correspsonds to the chunk at (32, 64, 96) assuming a chunk size of 32
pub type ChunkLoc = (i32, i32, i32);
pub type VoxelPos = (i8, i8, i8);

/// Convert world coordinate to a chunk location
#[inline]
pub fn wpos_to_locv(world_pos: Vector3) -> Vector3 {
	(world_pos / WIDTH_F).floor()
}

#[inline]
pub fn locv_to_wpos(locv: Vector3) -> Vector3 {
	locv * WIDTH_F
}

#[inline]
pub fn wpos_to_loc(world_pos: Vector3) -> ChunkLoc {
	locv_to_loc(wpos_to_locv(world_pos))
}

/// Convert Vector3 to i32 tuple
#[inline]
pub fn locv_to_loc(loc: Vector3) -> ChunkLoc {
	(loc.x as i32, loc.y as i32, loc.z as i32)
}

#[inline]
#[allow(unused)]
pub fn loc_to_wpos(loc: ChunkLoc) -> Vector3 {
	Vector3::new(loc.0 as f32, loc.1 as f32, loc.2 as f32) * WIDTH_F
}

#[inline]
pub fn loc_to_locv(loc: ChunkLoc) -> Vector3 {
	Vector3::new(loc.0 as f32, loc.1 as f32, loc.2 as f32)
}

/// convert world coordinate to a position within the chunk
#[inline]
pub fn wpos_to_vposv(world_pos: Vector3) -> Vector3 {
	world_pos.floor().posmod(WIDTH_F)
}

/// [i32] to [Vector3]
#[inline]
pub const fn ivec3(x: i32, y: i32, z: i32) -> Vector3 {
	Vector3::new(x as f32, y as f32, z as f32)
}

#[inline]
pub const fn uvec3(x: usize, y: usize, z: usize) -> Vector3 {
	Vector3::new(x as f32, y as f32, z as f32)
}

#[inline]
pub fn vposv_in_bounds(pos: Vector3) -> bool{
	pos.x >= 0.0 && pos.x < WIDTH_F &&
	pos.y >= 0.0 && pos.y < WIDTH_F &&
	pos.z >= 0.0 && pos.z < WIDTH_F
}

#[inline]
pub fn vpos_in_bounds(vpos: VoxelPos) -> bool{
	vpos.0 >= 0 && vpos.0 < WIDTH_I8 &&
	vpos.1 >= 0 && vpos.1 < WIDTH_I8 &&
	vpos.2 >= 0 && vpos.2 < WIDTH_I8
}

#[inline]
pub fn vpos_to_index(vpos: VoxelPos) -> usize {
	vpos.0 as usize * AREA
	+ vpos.1 as usize * WIDTH
	+ vpos.2 as usize
}

#[inline]
pub fn vposv_to_index(vposv: Vector3) -> usize {
	vposv.x as usize * AREA
	+ vposv.y as usize * WIDTH
	+ vposv.z as usize
}

#[inline]
#[allow(unused)]
pub fn index_to_vpos(i: usize) -> VoxelPos {
	(
		(i/AREA) as i8,
		(i/WIDTH % WIDTH) as i8,
		(i % WIDTH) as i8
	)
}

#[inline]
pub fn index_to_vposv(i: usize) -> Vector3 {
	Vector3::new(
		((i / AREA) as f32).floor(),
		((i/WIDTH % WIDTH) as f32).floor(),
		(i % WIDTH) as f32
	)
}

#[derive(ToVariant)]
pub struct Ray {
	hit: bool,
	pos: Vector3,
	normal: Vector3,
	voxel: Voxel,
	distance: f32,
}

impl Ray {
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
	fn is_air(&self) -> bool;
	fn is_surface(&self) -> bool;
	fn is_transparent(&self) -> bool;
	fn is_opaque(&self) -> bool;
}

impl VoxelData for Voxel {
	fn name(&self) -> String {
		match *self {
			0 => "air".into(),
			1 => "stone".into(),
			2 => "dirt".into(),
			3 => "grass".into(),
			4 => "frame".into(),
			5 => "leaves".into(),
			255 => "debug".into(),
			other => format!("{}", other),
		}
	}
	
	#[inline]
	fn is_air(&self) -> bool {
		*self == 0
	}

	#[inline]
	fn is_surface(&self) -> bool {
		*self != 0
	}

	#[inline]
	fn is_transparent(&self) -> bool {
		*self == 0
		|| *self == 4 
		|| *self == 5
	}

	#[inline]
	fn is_opaque(&self) -> bool {
		!self.is_transparent()
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
