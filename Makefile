default:
	cd gdvoxels && cargo build
	ln -sf ../../../../../gdvoxels/target/debug/libgdvoxels.so project/addons/voxel-engine/bin/linux/libgdvoxels.so

r: release
release:
	cd gdvoxels && cargo build --release
	ln -sf ../../../../../gdvoxels/target/release/libgdvoxels.so project/addons/voxel-engine/bin/linux/libgdvoxels.so

c: clippy
clippy:
	cd gdvoxels && cargo clippy
