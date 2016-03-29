//! A field defined by a translation of another field.

use cgmath::{Point, Point3, Vector3};

use field;

#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct T<Field> {
  pub translation: Vector3<f32>,
  pub field: Field,
}

impl<Field> field::T for T<Field> where Field: field::T {
  fn density(&mut self, p: &Point3<f32>) -> f32 {
    let p = p.add_v(&-self.translation);
    field::T::density(&mut self.field, &p)
  }

  fn normal(&mut self, p: &Point3<f32>) -> Vector3<f32> {
    let p = p.add_v(&-self.translation);
    field::T::normal(&mut self.field, &p)
  }
}
