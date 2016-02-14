//! This crate defines voxel data types and interfaces for interacting with voxels.

#![allow(let_and_return)]
#![allow(match_ref_pats)]
#![allow(type_complexity)]
#![allow(unneeded_field_pattern)]
#![allow(cyclomatic_complexity)]
#![allow(option_map_unwrap_or)]
#![deny(missing_docs)]
#![deny(warnings)]

#![feature(main)]
#![feature(plugin)]
#![feature(test)]
#![feature(unboxed_closures)]

#![plugin(clippy)]

extern crate cgmath;
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
    brush: &brush::T<Mosaic>,
  ) where Mosaic: mosaic::T<Material>;
}
