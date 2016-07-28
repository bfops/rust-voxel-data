//! Voxel brush module

use collision::Aabb3;

#[allow(missing_docs)]
pub type Bounds = Aabb3<i32>;

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct T<Mosaic> {
  /// The bounds of this brush.
  pub bounds: Bounds,
  /// The mosaic that this brush will apply.
  pub mosaic: Mosaic,
  /// lg of the smallest voxel size this brush will touch.
  pub min_lg_size: i16,
}

unsafe impl<Mosaic> Send for T<Mosaic> {}
