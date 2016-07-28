//! Translate another voxel mosaic.

use cgmath::{Point3, Vector3};

use field;
use mosaic;

#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct T<Mosaic> {
  pub translation: Vector3<f32>,
  pub mosaic: Mosaic,
}

impl<Mosaic> field::T for T<Mosaic> where Mosaic: field::T {
  fn density(&mut self, p: &Point3<f32>) -> f32 {
    let p = p + -self.translation;
    field::T::density(&mut self.mosaic, &p)
  }

  fn normal(&mut self, p: &Point3<f32>) -> Vector3<f32> {
    let p = p + -self.translation;
    field::T::normal(&mut self.mosaic, &p)
  }
}

impl<Mosaic, Material> mosaic::T<Material> for T<Mosaic> where Mosaic: mosaic::T<Material> {
  fn density(&mut self, p: &Point3<f32>) -> f32 {
    let p = p + -self.translation;
    mosaic::T::density(&mut self.mosaic, &p)
  }

  fn material(&mut self, p: &Point3<f32>) -> Option<Material> {
    let p = p + -self.translation;
    mosaic::T::material(&mut self.mosaic, &p)
  }
}
