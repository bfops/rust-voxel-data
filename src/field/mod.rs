//! A density field defining a density and normal everywhere.

use cgmath::{Point3, Vector3};
use std::ops::DerefMut;

pub mod sphere;
pub mod intersection;
pub mod rotation;
pub mod translation;

#[allow(missing_docs)]
pub trait T {
  /// The density of the material at this point.
  fn density(&mut self, p: &Point3<f32>) -> f32;

  /// The surface normal at a given point.
  fn normal(&mut self, p: &Point3<f32>) -> Vector3<f32>;
}

impl<X: ?Sized> T for Box<X> where X: T {
  fn density(&mut self, p: &Point3<f32>) -> f32 {
    T::density(self.deref_mut(), p)
  }

  fn normal(&mut self, p: &Point3<f32>) -> Vector3<f32> {
    T::normal(self.deref_mut(), p)
  }
}
