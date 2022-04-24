use gdnative::prelude::*;

mod world;
mod chunk;
mod common;
mod materials;

use world::*;

fn init(handle: InitHandle) {
	handle.add_class::<VoxelWorld>();
}

godot_init!(init);
