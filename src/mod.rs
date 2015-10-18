//! This crate defines voxel data types and interfaces for interacting with voxels.

#![allow(let_and_return)]
#![allow(match_ref_pats)]
#![allow(type_complexity)]
#![deny(missing_docs)]
#![deny(warnings)]

#![feature(iter_cmp)]
#![feature(main)]
#![feature(plugin)]
#![feature(test)]
#![feature(unboxed_closures)]

#![plugin(clippy)]

extern crate cgmath;
#[macro_use]
extern crate log;
extern crate test;

pub mod bounds;
pub mod field;
pub mod tree;

pub mod impls;
