//! A sphere field

use cgmath::{Point, Point3, Vector3, EuclideanVector};

use field;

#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct T {
  pub half_extents: Vector3<f32>,
}

unsafe impl Send for T {}

impl field::T for T {
  fn density(&self, p: &Point3<f32>) -> f32 {
    let v = self.half_extents - cgmath::Vector3::new(p.x.abs(), p.y.abs(), p.z.abs());
    v.x.min(v.y.min(v.z))
  }
}
