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

