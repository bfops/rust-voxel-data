[![Build Status](https://travis-ci.org/bfops/rust-voxel-data.svg?branch=master)](https://travis-ci.org/bfops/rust-voxel-data)

This library contains voxel storage modification code, meant to be reasonably performant and flexible. It's generic over the voxel type, with specific voxel 
implementations in `src/impls`.

It is by no means complete! Changes and additions are heartily encouraged, although some things (e.g. [mesh extraction](https://github.com/bfops/rust-isosurface-extraction))
might go better in separate libraries on top of this one.

## Sparse Voxel Octree

The `tree` module defines a sparse voxel octree (SVO) data structure. Every branch point in the tree can optionally contain a single voxel to represent that entire section,
so the same space can be stored at multiple levels of details.

## Fields, Mosaics & Brushes

The `field` modules defines a field trait to define a density and normal for every point in space. This is used to represent volumetric data.
The `mosaic` module extends fields with materials, such that any point in space can correspond to a material, or to nothing.
Mosaics are the basis for voxel brushes, which can be used to edit the SVO. Brushes provide the bounds of the brush, as well as a mosaic;
everywhere the mosaic defines a material, the corresponding voxel is overwritten.

A few basic fields and mosaics (such as spheres) are provided, and some "higher-order" types are provided to transform or combine simpler ones.
