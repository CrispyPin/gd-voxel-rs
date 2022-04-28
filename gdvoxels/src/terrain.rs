use gdnative::prelude::*;
use gdnative::api::OpenSimplexNoise;

use crate::chunk::core::*;
use crate::common::*;

type Noise = Ref<OpenSimplexNoise, Unique>;

pub struct TerrainGenerator {
	seed: i64,
	detail: Noise,
	mountain_mask: Noise,
	mountain: Noise,
	mountain_detail: Noise,
}

impl TerrainGenerator {
	pub fn new(seed: i64) -> Self {
		let mut instance = Self {
			seed,
			detail: OpenSimplexNoise::new(),
			mountain_mask: OpenSimplexNoise::new(),
			mountain: OpenSimplexNoise::new(),
			mountain_detail: OpenSimplexNoise::new(),
		};
		instance.setup();
		instance
	}

	fn setup(&mut self) {
		self.detail.set_seed(self.seed);
		self.mountain_mask.set_seed(self.seed);
		self.mountain.set_seed(self.seed);
		self.mountain_detail.set_seed(self.seed);
		
		self.detail.set_octaves(4);
		
		self.mountain_mask.set_octaves(1);
		self.mountain_mask.set_period(256.0);

		self.mountain.set_period(128.0);
		self.mountain.set_octaves(2);

		self.mountain_detail.set_period(32.0);
		self.mountain_detail.set_octaves(5);
	}

	pub fn generate(&self, pos: Vector3) -> ChunkCore {
		let mut new_core = ChunkCore::new();

		if pos.y > WIDTH_F * 4.0 {
			return new_core;
		}
		if pos.y < WIDTH_F * -4.0 {
			return new_core;
		}

		for x in 0..WIDTH {
			for z in 0..WIDTH {
				let world_x = x as f64 + pos.x as f64;
				let world_z = z as f64 + pos.z as f64;
				let height = self.height(world_x, world_z) as f32;
				for y in 0..WIDTH {
					let pos_y = y as f32 + pos.y;
					if  pos_y < height {
						new_core.set_voxel(Vector3::new(x as f32, y as f32, z as f32), 1);
					}
					else if pos_y < height + 2.0 {
						new_core.set_voxel(Vector3::new(x as f32, y as f32, z as f32), 2);
					}
					else if pos_y < height + 3.0 {
						new_core.set_voxel(Vector3::new(x as f32, y as f32, z as f32), 3);
					}
				}
			}
		}
		new_core
	}

	pub fn height(&self, x: f64, y: f64) -> f64 {
		self.detail.get_noise_2d(x, y) * 16.0 + 
		sigmoid(self.mountain_mask.get_noise_2d(x, y), 8.0)
			* (self.mountain.get_noise_2d(x, y) * 100.0
			+ self.mountain_detail.get_noise_2d(x, y) * 16.0)
	}
}

fn sigmoid(x: f64, k: f64) -> f64 {
	1.0 / (1.0 + std::f64::consts::E.powf(-k*x))
} 
