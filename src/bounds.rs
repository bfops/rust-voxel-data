//! Voxel bounds

use std::hash::{Hash, Hasher};
use std::mem;
use std::slice;

use cgmath::{Point, Point3, Vector3};

#[derive(Debug, Copy, Clone, PartialEq, Eq, RustcEncodable, RustcDecodable)]
#[allow(missing_docs)]
/// The input coordinates should be divided by (2^lg_size) relative to world coords.
pub struct T {
  pub x: i32,
  pub y: i32,
  pub z: i32,
  /// The log_2 of the voxel's size.
  pub lg_size: i16,
}

impl Hash for T {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    unsafe {
      let p: *const u8 = mem::transmute(self);
      state.write(slice::from_raw_parts(p, mem::size_of::<T>()))
    }
  }
}

#[allow(missing_docs)]
#[inline]
pub fn new(x: i32, y: i32, z: i32, lg_size: i16) -> T {
  let ret =
    T {
      x: x,
      y: y,
      z: z,
      lg_size: lg_size,
    };
  ret
}

impl T {
  /// The width of this voxel.
  #[inline]
  pub fn size(&self) -> f32 {
    if self.lg_size >= 0 {
      (1 << self.lg_size) as f32
    } else {
      1.0 / (1 << -self.lg_size) as f32
    }
  }

  /// The bottom of this voxel.
  #[inline]
  pub fn low_corner(&self) -> Point3<f32> {
    let size = self.size();
    Point3::new(self.x as f32, self.y as f32, self.z as f32).mul_s(size)
  }

  /// The corners of this voxel.
  #[inline]
  pub fn corners(&self) -> (Point3<f32>, Point3<f32>) {
    let size = self.size();
    let low = Point3::new(self.x as f32, self.y as f32, self.z as f32).mul_s(size);
    (low, low.add_v(&Vector3::new(size, size, size)))
  }

  /// The center of this voxel.
  #[inline]
  pub fn center(&self) -> Point3<f32> {
    let size = self.size();
    let half = Vector3::new(0.5, 0.5, 0.5);
    Point3::new(self.x as f32, self.y as f32, self.z as f32).add_v(&half).mul_s(size)
  }

  /// Check whether this voxel contains a given point.
  #[inline]
  pub fn contains_point(&self, p: &Point3<f32>) -> bool {
    let (low, high) = self.corners();
    p.x >= low.x &&
    p.y >= low.y &&
    p.z >= low.z &&
    p.x < high.x &&
    p.y < high.y &&
    p.z < high.z &&
    true
  }

  /// Check whether this voxel contains another one
  #[inline]
  pub fn contains(&self, other: &T) -> bool {
    if other.lg_size > self.lg_size {
      return false
    }

    // Convert `other`'s coordinates to be in terms of `self`'s lg_size.
    // This rounds down, so any voxels inside `self` will end up equal to self.
    let lg_ratio = self.lg_size - other.lg_size;
    (other.x >> lg_ratio) == self.x &&
    (other.y >> lg_ratio) == self.y &&
    (other.z >> lg_ratio) == self.z &&
    true
  }
}
