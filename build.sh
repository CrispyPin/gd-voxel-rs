#!/bin/bash

build_dev () {
	cd gdvoxels
	cargo build
	cd ..
	ln -sf ../../../../../gdvoxels/target/debug/libgdvoxels.so project/addons/voxel-engine/bin/linux/libgdvoxels.so
}


build_release () {
	cd gdvoxels
	cargo build --release
	cd ..
	ln -sf ../../../../../gdvoxels/target/release/libgdvoxels.so project/addons/voxel-engine/bin/linux/libgdvoxels.so
}

case $1 in
    r|release)
        build_release
        ;;
    *)
        build_dev
        ;;
esac