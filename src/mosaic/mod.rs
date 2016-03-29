//! A density field that also defines materials. This does not need to be defined everywhere.

use cgmath::{Point3};
use std::ops::DerefMut;

pub mod solid;
pub mod union;
pub mod translation;

use field;

#[allow(missing_docs)]
pub trait T<Material>: field::T {
  /// The material density at a given point. This should be nonnegative!
  fn density(&mut self, p: &Point3<f32>) -> f32 {
    field::T::density(self, p).abs()
  }

  /// The material at this point.
  fn material(&mut self, p: &Point3<f32>) -> Option<Material>;
}

impl<X: ?Sized, Material> T<Material> for Box<X> where X: T<Material> {
  fn density(&mut self, p: &Point3<f32>) -> f32 {
    T::density(self.deref_mut(), p)
  }

  fn material(&mut self, p: &Point3<f32>) -> Option<Material> {
    T::material(self.deref_mut(), p)
  }
}
