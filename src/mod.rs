//! This crate defines voxel data types and interfaces for interacting with voxels.

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(main)]
#![feature(test)]
#![feature(unboxed_closures)]

extern crate cgmath;
extern crate collision;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate test;

pub mod bounds;
pub mod brush;
pub mod field;
pub mod mosaic;
pub mod tree;

pub mod impls;

/// The interface provided by Voxels.
pub trait T<Material> {
  /// Apply a brush to this voxel.
  fn brush<Mosaic>(
    this: &mut Self,
    bounds: &bounds::T,
    brush: &mut brush::T<Mosaic>,
  ) where Mosaic: mosaic::T<Material>;
}
