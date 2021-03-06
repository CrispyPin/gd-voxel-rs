use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};
use gdnative::prelude::*;

use crate::chunk::core::ChunkCore;
use crate::common::*;
use crate::chunk::*;
use crate::materials::*;
use crate::terrain::*;


fn chunk_name(loc: ChunkLoc) -> String {
	format!("Chunk{:?}", loc)
}

enum MeshCommand {
	Generate(Chunk),
	Cancel(ChunkLoc),
	Exit,
}

/// Generate is a locv
enum GeneratorCommand {
	Generate(Vector3),
	Cancel(ChunkLoc),
	Exit,
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct VoxelWorld {
	#[property]
	load_distance: u16,
	#[property]
	auto_load: bool,
	#[property]
	max_chunks_loaded: u16,
	#[property]
	max_chunks_unloaded: u16,
	player_loc: Arc<Mutex<Vector3>>,
	chunks: HashMap<ChunkLoc, ChunkContainer>,
	unoptimised_chunks: Vec<ChunkLoc>,
	unload_queue: Vec<ChunkLoc>,
	materials: Arc<MaterialList>,


	gen_queue: Sender<GeneratorCommand>,
	mesh_queue: Sender<MeshCommand>,
	finished_chunks_recv: Receiver<Chunk>,
	mesh_thread_handle: Option<JoinHandle<()>>,
	gen_thread_handle: Option<JoinHandle<()>>,
}

enum ChunkContainer {
	Waiting,
	Ready(Chunk),
	Empty,
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
			unoptimised_chunks: Vec::new(),
			unload_queue: Vec::new(),
			load_distance: 2,
			max_chunks_loaded: 32,
			max_chunks_unloaded: 64,
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
		godot_print!("Exiting");
		self.gen_queue.send(GeneratorCommand::Exit).unwrap();
		self.mesh_queue.send(MeshCommand::Exit).unwrap();
		self.mesh_thread_handle.take().map(JoinHandle::join);
		self.gen_thread_handle.take().map(JoinHandle::join);
	}

	#[export]
	fn set_player_pos(&mut self, _owner: &Node, new_pos: Vector3) {
		let mut changed = false;
		let new_loc = wpos_to_locv(new_pos);
		{
			let mut player_loc = self.player_loc.lock().unwrap();
			if new_loc != *player_loc {
				*player_loc = new_loc;
				changed = true;
			}
		}
		if changed && self.auto_load {
			self.load_near();
			self.queue_unload_far();
		}
	}

	#[export]
	fn _process(&mut self, owner: &Node, _delta: f32) {
		self.collect_chunks(owner);

		self.unload();
		self.optimise_chunks();
	}

	/// casts ray through world, sees unloaded chunks as empty
	/// max_len is clamped to 0.001..65536.0
	#[export]
	fn cast_ray(&mut self, owner: &Node, source: Vector3, dir: Vector3, max_len: f32) -> Ray {
		let dir = dir.normalized();
		let max_len = max_len.clamp(0.001, 65536.0);
		let stepped_dir = step(0.0, dir);
		let mut ray_len = 0.0;
		
		while ray_len <= max_len {
			let ray_pos = source + dir * ray_len;
			let voxel = self.get_voxel(owner, ray_pos);
			if voxel != EMPTY {
				let normal = calc_normal(ray_pos);
				return Ray::hit(ray_pos, normal, voxel, ray_len);
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

		Ray::miss(source + dir * max_len, max_len)
	}

	#[export]
	fn set_voxel(&mut self, owner: &Node, wpos: Vector3, voxel: Voxel) {
		let loc = wpos_to_loc(wpos);
		
		if self.chunks.contains_key(&loc) {
			let materials = self.materials.clone();
			let chunkc = self.get_chunk_mut(loc).unwrap();
			let vposv = wpos_to_vposv(wpos);
			if chunkc.is_ready() {
				let old_voxel = chunkc.get_voxel(vposv);
				chunkc.set_voxel(vposv, voxel);
				chunkc.chunk_mut().unwrap().remesh_pos(&materials, vposv, old_voxel);
			}
			else if chunkc.is_empty() {
				let mut new_chunk = Chunk::new(wpos, ChunkCore::new());
				new_chunk.set_voxel(vposv, voxel);
				new_chunk.mark_empty(false);
				new_chunk.mesh_fast(&materials);
				self.spawn_chunk_node(owner, loc, &new_chunk);
				self.chunks.insert(loc, ChunkContainer::Ready(new_chunk));
			}
			if !self.unoptimised_chunks.contains(&loc) {
				self.unoptimised_chunks.push(loc);
			}
		}
	}

	#[export]
	fn get_voxel(&mut self, _owner: &Node, wpos: Vector3) -> Voxel {
		let loc = wpos_to_loc(wpos);
		let vpos = wpos_to_vposv(wpos);
		if self.chunk_is_loaded(loc) {
			return self.get_chunk(loc).unwrap().get_voxel(vpos)
		}
		EMPTY
	}

	#[export]
	fn chunk_count(&self, _owner: &Node) -> usize {
		self.chunks.len()
	}

	#[export]
	fn loaded_chunk_count(&self, _owner: &Node) -> usize {
		self.chunks.values().filter(|x| x.is_ready()).count()
	}

	#[export]
	fn empty_chunk_count(&self, _owner: &Node) -> usize {
		self.chunks.values().filter(|x| x.is_empty()).count()
	}

	#[export]
	fn waiting_chunk_count(&self, _owner: &Node) -> usize {
		self.chunks.values().filter(|x| x.is_waiting()).count()
	}

	/// load chunks around player pos
	fn load_near(&mut self) {
		let center_chunk = *self.player_loc.lock().unwrap();
		let radius = self.load_distance as i32;
		
		for x in  -radius..(radius + 1) {
			for y in  -radius..(radius + 1) {
				for z in  -radius..(radius + 1) {
					let loc = center_chunk + ivec3(x, y, z);
					self.load_or_generate(loc);
				}
			} 
		}
	}

	/// unload chunks far from player
	fn queue_unload_far(&mut self) {
		let player_loc = *self.player_loc.lock().unwrap();
		
		// let mut to_unload = Vec::new();
		let mut to_cancel = Vec::new();
		for (loc, chunk) in self.chunks.iter() {
			let delta = loc_to_locv(*loc) - player_loc;
			let delta = delta.abs();
			let dist = delta.x.max(delta.y).max(delta.z);
			if dist > self.load_distance as f32 + 1.0 {
				if chunk.is_ready() {
					self.unload_queue.push(*loc);
				}
				else {
					to_cancel.push(*loc);
				}
			}
		}
		for loc in to_cancel {
			self.chunks.remove(&loc);
			self.cancel_generation(loc);
		}
	}

	fn unload(&mut self) {
		let mut count = 0;
		while let Some(loc) = self.unload_queue.pop() {
			if self.chunk_is_loaded(loc) {
				unsafe {
					self.get_chunk(loc)
					.unwrap()
					.chunk()
					.unwrap()
					.node
					.assume_safe()
					.queue_free();
				}
				count += 1;
			}
			self.chunks.remove(&loc);
			if count > self.max_chunks_unloaded {
				break;
			}
		}
		
	}

	fn cancel_generation(&mut self, loc: ChunkLoc) {
		self.gen_queue.send(GeneratorCommand::Cancel(loc)).unwrap();
		self.mesh_queue.send(MeshCommand::Cancel(loc)).unwrap();
		self.chunks.remove(&loc);
	}

	fn optimise_chunks(&mut self) {
		let mut i = 0;
		while i < self.unoptimised_chunks.len() {
			let loc = self.unoptimised_chunks[i];
			if self.chunk_is_loaded(loc) {
				let materials = self.materials.clone();
				let chunk = self.get_chunk_mut(loc).unwrap().chunk_mut().unwrap();
				if chunk.since_change().elapsed().as_millis() >= 1000 {
					chunk.optimise(&materials);
					self.unoptimised_chunks.swap_remove(i);
				}
				else {
					i += 1;
				}
			} else {
				self.unoptimised_chunks.swap_remove(i);
			}
		}
	}

	/// if chunk at loc is not already loaded, generate a new one
	/// (todo) load from disk if it exists instead of generating
	fn load_or_generate(&mut self, locv: Vector3) {
		let loc = locv_to_loc(locv);
		if self.chunk_is_loaded(loc) || self.chunk_is_loading(loc) {
			return;
		}
		self.begin_generate_chunk(loc);
	}

	fn begin_generate_chunk(&mut self, loc: ChunkLoc) {
		if TerrainGenerator::loc_has_terrain(loc) {
			self.chunks.insert(loc, ChunkContainer::Waiting);
			self.gen_queue.send(GeneratorCommand::Generate(loc_to_locv(loc))).unwrap();
		}
		else {
			self.chunks.insert(loc, ChunkContainer::Empty);
		}
		if self.unload_queue.contains(&loc) {
			let mut i = 0;
			for j in 0..self.unload_queue.len() {
				if self.unload_queue[j] == loc {
					i = j;
					break;
				}
			}
			self.unload_queue.swap_remove(i);
		}
	}

	fn collect_chunks(&mut self, owner: &Node) {
		let mut count = 0;
		while let Ok(new_chunk) = self.finished_chunks_recv.try_recv() {
			let loc = locv_to_loc(new_chunk.wpos / WIDTH_F);

			if new_chunk.is_empty() {
				self.chunks.insert(loc, ChunkContainer::Empty);
				continue;
			}
			
			self.spawn_chunk_node(owner, loc, &new_chunk);
			self.chunks.insert(loc, ChunkContainer::Ready(new_chunk));
			count += 1;

			if count > self.max_chunks_loaded {
				break;
			}
		}
	}

	fn spawn_chunk_node(&mut self, owner: &Node, loc: ChunkLoc, new_chunk: &Chunk) {
		let mesh = unsafe {new_chunk.node.assume_safe()};
		mesh.set_mesh(new_chunk.array_mesh());
		mesh.set_translation(new_chunk.wpos);
		mesh.set_name(chunk_name(loc));
		let mesh = unsafe { mesh.assume_shared() };
		owner.add_child(mesh, false);
	}

	fn chunk_is_loaded(&self, loc: ChunkLoc) -> bool {
		if let Some(container) = self.chunks.get(&loc) {
			return container.is_ready();
		}
		false
	}

	fn chunk_is_loading(&self, loc: ChunkLoc) -> bool {
		if let Some(container) = self.chunks.get(&loc) {
			return container.is_waiting();
		}
		false
	}

	#[inline]
	fn get_chunk_mut(&mut self, loc: ChunkLoc) -> Option<&mut ChunkContainer> {
		self.chunks.get_mut(&loc)
	}
	
	#[inline]
	fn get_chunk(&self, loc: ChunkLoc) -> Option<&ChunkContainer> {
		self.chunks.get(&loc)
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

		'mainloop: loop {
			let mut recieved = if queue.is_empty() {
				// if queue is empty, block thread until more chunks are requested to save cpu
				gen_queue_recv.recv().ok()
			} else {
				gen_queue_recv.try_recv().ok()
			};
			while let Some(cmd) = recieved.take() {
				match cmd {
					GeneratorCommand::Exit => break 'mainloop,
					GeneratorCommand::Cancel(loc) => {
						let locv = loc_to_locv(loc);
						for i in 0..queue.len() {
							if queue[i] == locv {
								queue.remove(i);
								break;
							}
						}
					},
					GeneratorCommand::Generate(pos) => queue.push(pos),
				}
				recieved = gen_queue_recv.try_recv().ok();
			}
			if queue.is_empty() {continue;}
			// sort so closest chunk is at the end
			let player_loc = *player_pos.lock().unwrap();
			queue.sort_by(|a, b| a.distance_squared_to(player_loc).partial_cmp(&b.distance_squared_to(player_loc)).unwrap());
			
			let locv = queue.remove(0);
			let wpos = locv_to_wpos(locv);
			let new_chunk = Chunk::new(wpos, terrain_gen.generate(wpos));
			mesh_queue_terrain.send(MeshCommand::Generate(new_chunk)).unwrap();
		}
		godot_print!("Terrain thread exiting");
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
		
		'mainloop: loop {
			let mut recieved = if queue.is_empty() {
				// if queue is empty, block thread until more chunks are requested to save cpu
				mesh_queue_recv.recv().ok()
			} else {
				mesh_queue_recv.try_recv().ok()
			};
			while let Some(cmd) = recieved.take() {
				match cmd {
					MeshCommand::Exit => break 'mainloop,
					MeshCommand::Generate(chunk) => queue.push(chunk),
					MeshCommand::Cancel(loc) => {
						let locv = loc_to_locv(loc);
						for i in 0..queue.len() {
							if wpos_to_locv(queue[i].wpos) == locv {
								queue.remove(i);
								break;
							}
						}
					}
				}
				recieved = mesh_queue_recv.try_recv().ok();
			}
			if queue.is_empty() {continue;}
			// sort so closest chunk is at the end
			let player = *player_loc.lock().unwrap() * WIDTH_F;
			queue.sort_by(|a, b| a.wpos.distance_squared_to(player).partial_cmp(&b.wpos.distance_squared_to(player)).unwrap());

			let mut chunk = queue.remove(0);
			chunk.optimise(&materials);
			finished_chunks.send(chunk).unwrap();

		}
		godot_print!("Mesh thread exiting");
	}).unwrap()
}


impl ChunkContainer {
	fn chunk_mut(&mut self) -> Option<&mut Chunk> {
		match self {
			Self::Ready(chunk) => Some(chunk),
			_ => None,
		}
	}

	fn chunk(&self) -> Option<&Chunk> {
		match self {
			Self::Ready(chunk) => Some(chunk),
			_ => None,
		}
	}

	fn get_voxel(&self, vpos: Vector3) -> Voxel{
		match self {
			Self::Waiting => EMPTY,
			Self::Ready(chunk) => chunk.get_voxel(vpos),
			Self::Empty => EMPTY,
		}
	}
	
	fn set_voxel(&mut self, vpos: Vector3, voxel: Voxel) {
		if let Self::Ready(chunk) = self {
			chunk.set_voxel(vpos, voxel);
		}
	}

	fn is_ready(&self) -> bool {
		if let Self::Ready(_) = self {
			return true;
		}
		false
	}

	fn is_empty(&self) -> bool {
		if let Self::Empty = self {
			return true;
		}
		false
	}

	fn is_waiting(&self) -> bool {
		if let Self::Waiting = self {
			return true;
		}
		false
	}
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
