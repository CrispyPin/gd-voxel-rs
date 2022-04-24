# gd-voxel-rs
A WIP voxel system for Godot.

Partially a rewrite of https://github.com/CrispyPin/voxel-meshing

**Todo**:
- mesh generation optimisation (in place editing)
- load chunks in a separate thread, closest first
- collision/player controller
- greedy meshing
- handle voxels with alpha
- terrain gen

**Plan**:
mesh in-place editing:
- for each material type:
	- find affected quads by searching entire chunk
	- remove them by moving quads from the end of the list to their place
	- add new quads
- transfer all mesh data

mesh optimisation:
- when chunk is stale for some seconds:
- start a thread for greedy mesh
- mark as unchanged
- copy voxel data
- greedy mesh
- apply greedy mesh if still unchanged
- keep fast mesh in memory and revert to it if a change happens

normals and UVs can be derived in the shader, storing voxel type in the mesh is not neccesary if each material type has a separate surface

voxel types that have different textures on different sides (eg grass) can be handled in the grass material


