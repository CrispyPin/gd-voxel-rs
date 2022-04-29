# gd-voxel-rs
A WIP voxel system for Godot.

spiritual successor to https://github.com/CrispyPin/voxel-meshing

**Todo**:
- only re-apply surfaces that were affected by a partial refresh
- collision/player controller
- greedy meshing
- handle voxels with alpha
- terrain gen

**mesh optimisation**:  
- when chunk is stale for some seconds:
- start a thread for greedy mesh
- mark as unchanged
- copy voxel data
- greedy mesh
- apply greedy mesh if still unchanged
- keep fast mesh in memory and revert to it if a change happens

# structure
- chunks: Hashmap<ChunkLoc, ChunkContainer>
	- ChunkLoc = (i32, i32, i32)
	- Option<Chunk>
		- mesh: ChunkMesh
		- core: ChunkCore
			- voxels: Box<[Voxel, VOLUME]>
		- node: Ref<MeshInstance, Shared>
		- wpos: Vector3
		- loc: ChunkLoc

# naming:
wpos = Vector3; world space coordinate
loc = ChunkLoc; i32 tuple for chunk location
locv = Vector3; chunk location in vector form (should be floored)
vpos = Vector3; position in local chunk space as Vector3, not necessarily floored
