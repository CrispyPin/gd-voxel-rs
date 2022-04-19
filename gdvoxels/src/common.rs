pub const WIDTH: usize = 32;
pub const AREA: usize = WIDTH * WIDTH;
pub const VOLUME: usize = AREA * WIDTH;

pub type Voxel = u8;
pub type ChunkData = [Voxel; VOLUME];

pub const EMPTY: Voxel = 0;
