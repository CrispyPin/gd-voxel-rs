use gdnative::prelude::*;

mod world;
mod chunk;
mod common;

use chunk::*;
use world::*;

fn init(handle: InitHandle) {
	handle.add_class::<Chunk>();
	handle.add_class::<VoxelWorld>();
}

godot_init!(init);
