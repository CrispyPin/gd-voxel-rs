use gdnative::prelude::*;

pub const WIDTH: usize = 32;
pub const AREA: usize = WIDTH * WIDTH;
pub const VOLUME: usize = AREA * WIDTH;

pub type Voxel = u8;

pub const EMPTY: Voxel = 0;

/// convert world coordinate to a chunk location
pub fn chunk_pos(world_pos: Vector3) -> Vector3 {
	(world_pos / WIDTH as f32).floor()
}

/// [i32] to [Vector3]
pub const fn ivec3(x: i32, y: i32, z: i32) -> Vector3 {
	Vector3::new(x as f32, y as f32, z as f32)
}

