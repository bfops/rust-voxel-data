//! A sphere field

use cgmath::{Point3, Vector3, EuclideanSpace, InnerSpace};

use field;

#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct T {
  pub radius: f32,
}

unsafe impl Send for T {}

impl field::T for T {
  fn density(&mut self, p: &Point3<f32>) -> f32 {
    self.radius*self.radius - p.to_vec().magnitude2()
  }

  fn normal(&mut self, p: &Point3<f32>) -> Vector3<f32> {
    p.to_vec().normalize()
  }
}
