//! A voxel implementation where each voxel stores a single mesh vertex and normal.

use cgmath::{Point, Point3, Vector, EuclideanVector, Vector3};
use std::cmp::{min, max};
use std::ops::Neg;

use bounds;

// NOTE: When voxel size and storage become an issue, this should be shrunk to
// be less than pointer-sized. It'll be easier to transfer to the GPU for
// whatever reasons, but also make it possible to shrink the SVO footprint by
// "flattening" the leaf contents and pointers into the same space (the
// low-order bits can be used to figure out which one it is, since pointers
// have three low-order bits set to zero).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum T<Material> {
  /// The entire voxel is a single material.
  Volume(Material),
  /// The voxel crosses the surface of the volume.
  Surface(SurfaceStruct<Material>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
// Every voxel keeps track of a single vertex, as well as whether its
// lowest-coordinate corner is inside the volume.
// Since we keep track of an "arbitrarily" large world of voxels, we don't
// leave out any corners.
#[allow(missing_docs)]
pub struct SurfaceStruct<Material> {
  /// The position of a free-floating vertex on the surface.
  pub surface_vertex: Vertex,
  /// The surface normal at `surface_vertex`.
  pub normal: Normal,

  /// The material of the voxel's lowest corner.
  pub corner: Material,
}

#[allow(missing_docs)]
pub fn unwrap<X>(voxel: T<Option<X>>) -> T<X> {
  match voxel {
    T::Volume(x) => T::Volume(x.unwrap()),
    T::Surface(x) => {
      T::Surface(SurfaceStruct {
        surface_vertex: x.surface_vertex,
        normal: x.normal,
        corner: x.corner.unwrap(),
      })
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
/// Vertex expressed using a fraction between voxel bounds.
pub struct Vertex {
  pub x: Fracu8,
  pub y: Fracu8,
  pub z: Fracu8,
}

impl Vertex {
  /// Given a voxel, convert this vertex to a world position.
  pub fn to_world_vertex(&self, parent: &bounds::T) -> Point3<f32> {
    // Relative position of the vertex.
    let local =
      Vector3::new(
        self.x.numerator as f32,
        self.y.numerator as f32,
        self.z.numerator as f32,
      )
      .div_s(256.0)
    ;
    let fparent = Point3::new(parent.x as f32, parent.y as f32, parent.z as f32);
    fparent.add_v(&local).mul_s(parent.size())
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
/// A compressed normal format.
pub struct Normal {
  pub x: Fraci8,
  pub y: Fraci8,
  pub z: Fraci8,
}

impl Normal {
  /// Turn a normalized floating-point normal into a packed format.
  pub fn of_float_normal(normal: &Vector3<f32>) -> Normal {
    // Okay, so we scale the normal by 127, and use 127 to represent 1.0.
    // Then we store it in a `Fraci8`, which scales by 128 and represents a
    // fraction in [-1,1). That seems wrong, but this is normal data, so scaling
    // doesn't matter. Sketch factor is over 9000, but it's not wrong.

    let normal = normal.mul_s(127.0);
    let normal = Vector3::new(normal.x as i32, normal.y as i32, normal.z as i32);
    Normal {
      x: Fraci8::of(max(-127, min(127, normal.x)) as i8),
      y: Fraci8::of(max(-127, min(127, normal.y)) as i8),
      z: Fraci8::of(max(-127, min(127, normal.z)) as i8),
    }
  }

  /// Convert from a packed format to a normalized floating-point normal.
  pub fn to_float_normal(&self) -> Vector3<f32> {
    Vector3::new(self.x.to_f32(), self.y.to_f32(), self.z.to_f32()).normalize()
  }
}

impl Neg for Normal {
  type Output = Normal;

  fn neg(self) -> Normal {
    Normal {
      x: Fraci8::of(-max(-127, self.x.numerator)),
      y: Fraci8::of(-max(-127, self.y.numerator)),
      z: Fraci8::of(-max(-127, self.z.numerator)),
    }
  }
}

/// Express a `[0,1)` fraction using a `u8`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Fracu8 {
  /// The numerator of a fraction over 1 << 8.
  pub numerator: u8,
}

impl Fracu8 {
  #[allow(missing_docs)]
  pub fn of(numerator: u8) -> Fracu8 {
    Fracu8 {
      numerator: numerator,
    }
  }
}

/// Express a `[-1,1)` fraction using a `i8`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Fraci8 {
  /// The numerator of a fraction over 1 << 8.
  pub numerator: i8,
}

impl Fraci8 {
  #[allow(missing_docs)]
  pub fn of(numerator: i8) -> Fraci8 {
    Fraci8 {
      numerator: numerator,
    }
  }

  #[allow(missing_docs)]
  pub fn to_f32(&self) -> f32 {
    self.numerator as f32 / 128.0
  }
}
