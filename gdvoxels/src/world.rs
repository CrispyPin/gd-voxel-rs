use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};
use gdnative::prelude::*;
use gdnative::api::MeshInstance;

use crate::common::*;
use crate::chunk::*;
use crate::materials::*;
use crate::terrain::*;

/// How often the terrain and mesh threads re-sort their queues by distance to player
/// 1 or 2 works best, higher values end up with a lot of fragmentation when moving fast
const CHUNK_QUEUE_SORT_FREQ: u8 = 2;

/// Represents a chunk location
/// 
/// Loc(1,2,3) correspsonds to the chunk at (32, 64, 96) assuming a chunk size of 32
type Loc = (i32, i32, i32);

enum MeshCommand {
	Generate(Chunk),
	Exit,
}

enum GeneratorCommand {
	Generate(Vector3),
	Exit,
}


#[derive(NativeClass)]
#[inherit(Node)]
pub struct VoxelWorld {
	#[property]
	load_distance: u16,
	#[property]
	auto_load: bool,
	player_loc: Arc<Mutex<Vector3>>,
	chunks: HashMap<Loc, Chunk>,
	materials: Arc<MaterialList>,

	chunks_in_progress: Vec<Loc>,
	gen_queue: Sender<GeneratorCommand>,
	mesh_queue: Sender<MeshCommand>,
	finished_chunks_recv: Receiver<Chunk>,
	mesh_thread_handle: Option<JoinHandle<()>>,
	gen_thread_handle: Option<JoinHandle<()>>,
}


#[methods]
impl VoxelWorld {
	fn new(_owner: &Node) -> Self {
		let (gen_queue, gen_queue_recv) = mpsc::channel();
		let (finished_chunks, finished_chunks_recv) = mpsc::channel();
		let (mesh_queue, mesh_queue_recv) = mpsc::channel();

		let player_loc = Arc::new(Mutex::new(Vector3::ZERO));
		let materials = Arc::new(MaterialList::new());
		let gen_thread_handle = terrain_thread(gen_queue_recv, mesh_queue.clone(), player_loc.clone());
		let mesh_thread_handle = mesh_thread(materials.clone(), mesh_queue_recv, finished_chunks, player_loc.clone());

		Self {
			chunks: HashMap::new(),
			chunks_in_progress: Vec::new(),
			load_distance: 2,
			auto_load: true,
			player_loc,
			gen_queue,
			finished_chunks_recv,
			mesh_queue,
			materials,
			gen_thread_handle: Some(gen_thread_handle),
			mesh_thread_handle: Some(mesh_thread_handle),
		}
	}

	#[export]
	fn _ready(&mut self, owner: TRef<Node>) {
		self.load_near();
		owner.connect("tree_exiting", owner, "_quit", VariantArray::new_shared(), 0).unwrap();
	}

	#[export]
	fn _quit(&mut self, _owner: &Node) {
		godot_print!("exiting!");
		self.gen_queue.send(GeneratorCommand::Exit).unwrap();
		self.mesh_queue.send(MeshCommand::Exit).unwrap();
		self.mesh_thread_handle.take().map(JoinHandle::join);
		self.gen_thread_handle.take().map(JoinHandle::join);
	}

	#[export]
	fn set_player_pos(&mut self, _owner: &Node, new_pos: Vector3) {
		let new_loc = chunk_loc(new_pos);
		let mut loc = self.player_loc.lock().unwrap();
		*loc = new_loc;
	}

	#[export]
	fn _process(&mut self, owner: &Node, _delta: f32) {
		if self.auto_load {
			self.load_near();
		}

		self.collect_chunks(owner);
	}

	/// casts ray through world, sees unloaded chunks as empty
	/// max_len is clamped to 0.001..65536.0
	#[export]
	fn cast_ray(&mut self, owner: &Node, source: Vector3, dir: Vector3, max_len: f32) -> RayResult {
		let dir = dir.normalized();
		let max_len = max_len.clamp(0.001, 65536.0);
		let stepped_dir = step(0.0, dir);
		let mut ray_len = 0.0;
		
		while ray_len <= max_len {
			let ray_pos = source + dir * ray_len;
			let voxel = self.get_voxel(owner, ray_pos);
			if voxel != EMPTY {
				let normal = calc_normal(ray_pos);
				return RayResult::hit(ray_pos, normal, voxel, ray_len);
			}
			
			let pos_in_voxel = fract(ray_pos);
			// distance "forward" along each axis to the next plane intersection
			let dist_to_next_plane = stepped_dir - pos_in_voxel;
			// distance along the ray to next plane intersection for each axis
			let deltas = dist_to_next_plane / dir;
			// move the smallest of these distances, so that no voxel is skipped
			ray_len += mincomp(deltas).max(0.0001);
		}

		fn calc_normal(hit_pos: Vector3) -> Vector3 {
			let pos_in_voxel = fract(hit_pos);
			let centered = pos_in_voxel - Vector3::ONE*0.5;
			let axis = centered.abs().max_axis();
			axis.vec() * centered.sign()
		}

		RayResult::miss(source + dir * max_len, max_len)
	}

	#[export]
	fn set_voxel(&mut self, _owner: &Node, pos: Vector3, voxel: Voxel) {
		let loc = chunk_loc(pos);
		if self.chunk_is_loaded(loc) {
			let materials = self.materials.clone();
			let chunk = self.get_chunk(loc).unwrap();
			let pos = local_pos(pos);
			let old_voxel = chunk.get_voxel(pos);
			chunk.set_voxel(pos, voxel);
			chunk.remesh_pos(&materials, pos, old_voxel);
		}
	}

	#[export]
	fn get_voxel(&mut self, _owner: &Node, pos: Vector3) -> Voxel{
		let loc = chunk_loc(pos);
		if let Some(chunk) = self.get_chunk(loc) {
			chunk.get_voxel(local_pos(pos))
		}
		else {
			EMPTY
		}
	}

	/// load chunks around player pos
	fn load_near(&mut self) {
		let center_chunk = *self.player_loc.lock().unwrap();
		
		// bad way of doing increasing cubes for loading
		for radius in 0..(self.load_distance as i32 + 1) {
			let range = -radius..(radius + 1);
			for x in range.clone() {
				for y in range.clone() {
					for z in range.clone() {
						let loc = center_chunk + ivec3(x, y, z);
						self.load_or_generate(loc);
					}
				} 
			}
		}
	}

	/// if chunk at loc is not already loaded, generate a new one
	/// (todo) load from disk if it exists instead of generating
	fn load_or_generate(&mut self, loc: Vector3) {
		if self.chunk_is_loaded(loc) || self.chunk_is_loading(loc) {
			return;
		}
		self.begin_create_chunk(loc);
	}

	fn begin_create_chunk(&mut self, loc: Vector3) {
		// godot_print!("creating {:?}", loc);
		self.chunks_in_progress.push(key(loc));
		self.gen_queue.send(GeneratorCommand::Generate(loc)).unwrap();
	}

	fn collect_chunks(&mut self, owner: &Node) {
		while let Ok(new_chunk) = self.finished_chunks_recv.try_recv() {
			let k = key(new_chunk.position / WIDTH_F);
			// godot_print!("Chunk Generated! {:?}", k);

			let mesh = MeshInstance::new();
			mesh.set_mesh(new_chunk.get_mesh());
			mesh.set_translation(new_chunk.position);
			mesh.set_name(format!("Chunk{:?}", k));
			owner.add_child(mesh, false);
			
			self.chunks.insert(k, new_chunk);
			for i in 0..self.chunks_in_progress.len() {
				if self.chunks_in_progress[i] == k {
					self.chunks_in_progress.remove(i);
					break;
				}
			}
		}
	}

	fn chunk_is_loaded(&self, loc: Vector3) -> bool {
		self.chunks.contains_key(&key(loc))
	}

	fn chunk_is_loading(&self, loc: Vector3) -> bool {
		self.chunks_in_progress.contains(&key(loc))
	}

	#[inline]
	fn get_chunk(&mut self, loc: Vector3) -> Option<&mut Chunk> {
		self.chunks.get_mut(&key(loc))
	}
}

fn terrain_thread(
	gen_queue_recv: Receiver<GeneratorCommand>,
	mesh_queue_terrain: Sender<MeshCommand>,
	player_pos: Arc<Mutex<Vector3>>
) -> JoinHandle<()> {
	thread::Builder::new().name("terrain".to_string()).spawn(move || {
		let terrain_gen = TerrainGenerator::new(42);
		let mut queue = Vec::new();
		let mut since_sorting = CHUNK_QUEUE_SORT_FREQ;
		'mainloop: loop {
			while let Ok(cmd) = gen_queue_recv.try_recv() {
				match cmd {
					GeneratorCommand::Exit => break 'mainloop,
					GeneratorCommand::Generate(pos) => queue.push(pos),
				}
			}
			if since_sorting == CHUNK_QUEUE_SORT_FREQ {
				since_sorting = 0;
				// sort so closest chunk is at the end
				let player_loc = *player_pos.lock().unwrap();
				queue.sort_by(|a, b| b.distance_squared_to(player_loc).partial_cmp(&a.distance_squared_to(player_loc)).unwrap());
			}
			since_sorting += 1;

			if let Some(loc) = queue.pop() {
				let pos = loc * WIDTH_F;
				let new_chunk = Chunk::new(pos, terrain_gen.generate(pos));
				mesh_queue_terrain.send(MeshCommand::Generate(new_chunk)).unwrap();
			}
		}
	}).unwrap()
}

fn mesh_thread(
	materials: Arc<MaterialList>,
	mesh_queue_recv: Receiver<MeshCommand>,
	finished_chunks: Sender<Chunk>,
	player_loc: Arc<Mutex<Vector3>>
) -> JoinHandle<()>{
	thread::Builder::new().name("mesh".to_string()).spawn(move || {
		let mut queue = Vec::new();
		let mut since_sorting = CHUNK_QUEUE_SORT_FREQ;
		'mainloop: loop {
			while let Ok(cmd) = mesh_queue_recv.try_recv() {
				match cmd {
					MeshCommand::Exit => break 'mainloop,
					MeshCommand::Generate(chunk) => queue.push(chunk),
				}
			}
			if since_sorting == CHUNK_QUEUE_SORT_FREQ {
				since_sorting = 0;
				// sort so closest chunk is at the end
				let player = *player_loc.lock().unwrap() * WIDTH_F;
				queue.sort_by(|a, b| b.position.distance_squared_to(player).partial_cmp(&a.position.distance_squared_to(player)).unwrap());
			}
			since_sorting += 1;

			if let Some(mut chunk) = queue.pop() {
				chunk.remesh(&materials);
				finished_chunks.send(chunk).unwrap();
			}
		}
	}).unwrap()
}


/// convert Vector3 to i32 tuple to use as a key in chunk array
#[inline]
fn key(loc: Vector3) -> Loc {
	let loc = loc.floor();
	(loc.x as i32, loc.y as i32, loc.z as i32)
}

#[inline]
fn fract(v: Vector3) -> Vector3 {
	Vector3::new(
		(v.x.fract() + 1.0).fract(),
		(v.y.fract() + 1.0).fract(),
		(v.z.fract() + 1.0).fract()
	)
}

#[inline]
fn mincomp(v: Vector3) -> f32 {
	v.x.min(v.y.min(v.z))
}

fn step(e: f32, v: Vector3) -> Vector3 {
	Vector3::new(
		(v.x >= e) as u8 as f32, 
		(v.y >= e) as u8 as f32, 
		(v.z >= e) as u8 as f32,
	)
}
