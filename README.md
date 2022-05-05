# gd-voxel-rs
A WIP voxel system for Godot.

spiritual successor to https://github.com/CrispyPin/voxel-meshing

# Todo:
- improve terrain generation
- collision/player controller

## Naming:
- wpos = Vector3; world space coordinate
- loc = ChunkLoc; i32 tuple for chunk location
- locv = Vector3; chunk location in vector form (should be floored)
- vpos = VoxelPos; i8 tuple for position in local chunk space
- vposv = Vector3; position in local chunk space as Vector3, not necessarily floored
