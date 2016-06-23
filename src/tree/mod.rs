//! Voxel octree

use cgmath::{Aabb, Point, Vector, Vector3, Ray3};
use std;

mod raycast;
pub mod traversal;

use brush;
use bounds;
use mosaic;

#[derive(Debug, RustcEncodable, RustcDecodable)]
/// A voxel octree; a voxel stored at a given level is the size of the entire subtree.
pub struct T<Voxel> {
  /// The tree extends 2^lg_size in each direction.
  /// i.e. the total width is 2^(lg_size + 1).
  pub lg_size: u8,
  /// Force the top level to always be branches;
  /// it saves a branch in the grow logic.
  pub contents: Branches<Voxel>,
}

#[derive(Debug, PartialEq, Eq, RustcEncodable, RustcDecodable)]
#[allow(missing_docs)]
#[repr(C)]
pub struct Branches<Voxel> {
  pub data: Option<Voxel>,

  // xyz ordering
  // This isn't an array because we can't move out of an array.

  lll: Inner<Voxel>,
  llh: Inner<Voxel>,
  lhl: Inner<Voxel>,
  lhh: Inner<Voxel>,
  hll: Inner<Voxel>,
  hlh: Inner<Voxel>,
  hhl: Inner<Voxel>,
  hhh: Inner<Voxel>,
}

/// The main, recursive, tree-y part of the voxel tree.
#[derive(Debug, PartialEq, Eq, RustcEncodable, RustcDecodable)]
#[allow(missing_docs)]
pub enum Inner<Voxel> {
  Empty,
  Branches(Box<Branches<Voxel>>),
}

impl<Voxel> Branches<Voxel> {
  #[allow(missing_docs)]
  pub fn empty() -> Branches<Voxel> {
    Branches {
      data: None,
      lll: Inner::Empty,
      llh: Inner::Empty,
      lhl: Inner::Empty,
      lhh: Inner::Empty,
      hll: Inner::Empty,
      hlh: Inner::Empty,
      hhl: Inner::Empty,
      hhh: Inner::Empty,
    }
  }

  #[allow(missing_docs)]
  pub fn as_flat_array(&self) -> &[Inner<Voxel>; 8] {
    unsafe {
      std::mem::transmute(&self.lll)
    }
  }

  #[allow(missing_docs)]
  pub fn as_flat_array_mut(&mut self) -> &mut [Inner<Voxel>; 8] {
    unsafe {
      std::mem::transmute(&mut self.lll)
    }
  }

  #[allow(missing_docs)]
  pub fn as_array(&self) -> &[[[Inner<Voxel>; 2]; 2]; 2] {
    unsafe {
      std::mem::transmute(&self.lll)
    }
  }

  #[allow(missing_docs)]
  pub fn as_array_mut(&mut self) -> &mut [[[Inner<Voxel>; 2]; 2]; 2] {
    unsafe {
      std::mem::transmute(&mut self.lll)
    }
  }
}

fn brush_overlaps(voxel: &bounds::T, brush: &brush::Bounds) -> bool {
  if voxel.lg_size >= 0 {
    let min =
      Vector3::new(
        voxel.x << voxel.lg_size,
        voxel.y << voxel.lg_size,
        voxel.z << voxel.lg_size,
      );
    min.x < brush.max().x &&
    min.y < brush.max().y &&
    min.z < brush.max().z &&
    {
      let max = min.add_s(1 << voxel.lg_size);
      brush.min().x < max.x &&
      brush.min().y < max.y &&
      brush.min().z < max.z &&
      true
    }
  } else {
    let lg_size = -voxel.lg_size;
    let max =
      Vector3::new(
        brush.max().x << lg_size,
        brush.max().y << lg_size,
        brush.max().z << lg_size,
      );
    voxel.x < max.x &&
    voxel.y < max.y &&
    voxel.z < max.z &&
    {
      let min =
        Vector3::new(
          brush.min().x << lg_size,
          brush.min().y << lg_size,
          brush.min().z << lg_size,
        );
      min.x <= voxel.x &&
      min.y <= voxel.y &&
      min.z <= voxel.z &&
      true
    }
  }
}

impl<Voxel> Inner<Voxel> {
  /// Create a tree leaf.
  pub fn leaf(voxel: Option<Voxel>) -> Inner<Voxel> {
    let mut branches = Branches::empty();
    branches.data = voxel;
    Inner::Branches(Box::new(branches))
  }

  /// Return the `Branches` data from this subtree. If none exists, create empty branch data.
  pub fn force_branches(&mut self) -> &mut Branches<Voxel> {
    match self {
      &mut Inner::Branches(ref mut branches) => branches,

      &mut Inner::Empty => {
        *self = Self::leaf(None);

        match self {
          &mut Inner::Branches(ref mut branches) => branches,
          _ => unreachable!(),
        }
      },
    }
  }

  #[allow(missing_docs)]
  pub fn voxel(&self) -> Option<&Voxel> {
    match self {
      &Inner::Branches(ref branches) => branches.data.as_ref(),
      &Inner::Empty => None,
    }
  }

  #[allow(missing_docs)]
  pub fn voxel_mut(&mut self) -> Option<&mut Voxel> {
    match self {
      &mut Inner::Branches(ref mut branches) => branches.data.as_mut(),
      &mut Inner::Empty => None,
    }
  }

  #[allow(missing_docs)]
  pub fn brush<Material, Mosaic, Generate, OnVoxelUpdate>(
    &mut self,
    bounds: &bounds::T,
    brush: &mut brush::T<Mosaic>,
    generate: &mut Generate,
    on_voxel_update: &mut OnVoxelUpdate,
  ) where
    Mosaic: mosaic::T<Material>,
    Voxel: ::T<Material>,
    Generate: FnMut(&::bounds::T) -> Option<Voxel>,
    OnVoxelUpdate: FnMut(&Voxel, &::bounds::T),
  {
    debug!("brush considers {:?}", bounds);
    if !brush_overlaps(bounds, &brush.bounds) {
      debug!("ignoring {:?}", bounds);
      return
    }

    if bounds.lg_size < brush.min_lg_size {
      return
    }

    let mut on_branches = |branches: &mut Box<Branches<Voxel>>| {
      match branches.data {
        None => {
          match generate(bounds) {
            None => {},
            Some(mut voxel) => {
              ::T::brush(&mut voxel, bounds, brush);
              on_voxel_update(&voxel, bounds);
              branches.data = Some(voxel);
            },
          }
        },
        Some(ref mut voxel) => {
          ::T::brush(voxel, bounds, brush);
          on_voxel_update(&voxel, bounds);
        },
      }

      // Bounds of the lowest branch
      let bounds = bounds::new(bounds.x << 1, bounds.y << 1, bounds.z << 1, bounds.lg_size - 1);

      macro_rules! recurse(($branch: ident, $update_bounds: expr) => {{
        let mut bounds = bounds;
        $update_bounds(&mut bounds);
        branches.$branch.brush(&bounds, brush, generate, on_voxel_update);
      }});
      recurse!(lll, |_|                 {                            });
      recurse!(llh, |b: &mut bounds::T| {                    b.z += 1});
      recurse!(lhl, |b: &mut bounds::T| {          b.y += 1          });
      recurse!(lhh, |b: &mut bounds::T| {          b.y += 1; b.z += 1});
      recurse!(hll, |b: &mut bounds::T| {b.x += 1                    });
      recurse!(hlh, |b: &mut bounds::T| {b.x += 1;           b.z += 1});
      recurse!(hhl, |b: &mut bounds::T| {b.x += 1; b.y += 1          });
      recurse!(hhh, |b: &mut bounds::T| {b.x += 1; b.y += 1; b.z += 1});
    };

    match self {
      &mut Inner::Branches(ref mut branches) => {
        on_branches(branches);
      },
      &mut Inner::Empty => {
        let mut branches = Box::new(Branches::empty());
        on_branches(&mut branches);
        *self = Inner::Branches(branches);
      },
    }
  }
}

#[allow(missing_docs)]
pub fn new<Voxel>() -> T<Voxel> {
  T {
    lg_size: 0,
    contents: Branches::<Voxel>::empty(),
  }
}

impl<Voxel> T<Voxel> {
  /// Is this voxel (non-strictly) within an origin-centered voxel with
  /// width `2^(lg_size + 1)`?
  pub fn contains_bounds(&self, voxel: &bounds::T) -> bool {
    let high;
    if voxel.lg_size >= 0 {
      high = (1 << self.lg_size) >> voxel.lg_size;
    } else {
      high = (1 << self.lg_size) << (-voxel.lg_size);
    }

    voxel.x < high &&
    voxel.y < high &&
    voxel.z < high &&
    {
      let low = -high;
      voxel.x >= low &&
      voxel.y >= low &&
      voxel.z >= low &&
      true
    }
  }

  /// Ensure that this tree can hold the provided voxel.
  pub fn grow_to_hold(&mut self, voxel: &bounds::T) {
    while !self.contains_bounds(voxel) {
      // Double the bounds in every direction.
      self.lg_size += 1;

      // Pull out `self.contents` so we can move out of it.
      let contents = std::mem::replace(&mut self.contents, Branches::<Voxel>::empty());

      // We re-construct the tree with bounds twice the size (but still centered
      // around the origin) by deconstructing the top level of branches,
      // creating a new doubly-sized top level, and moving the old branches back
      // in as the new top level's children. e.g. in 2D:
      //
      //                      ---------------------------
      //                      |     |     |0|     |     |
      //                      |     |     |0|     |     |
      // ---------------      ------------|0|------------
      // |  1  |0|  2  |      |     |  1  |0|  2  |     |
      // |     |0|     |      |     |     |0|     |     |
      // |------0------|      |------------0------------|
      // 000000000000000  ==> |0000000000000000000000000|
      // |------0------|      |------------0------------|
      // |     |0|     |      |     |     |0|     |     |
      // |  3  |0|  4  |      |     |  3  |0|  4  |     |
      // ---------------      |------------0------------|
      //                      |     |     |0|     |     |
      //                      |     |     |0|     |     |
      //                      ---------------------------

      macro_rules! at(
        ($c_idx:ident, $b_idx:ident) => {{
          let mut branches = Branches::<Voxel>::empty();
          branches.$b_idx = contents.$c_idx;
          Inner::Branches(Box::new(branches))
        }}
      );

      self.contents =
        Branches {
          data: None,
          lll: at!(lll, hhh),
          llh: at!(llh, hhl),
          lhl: at!(lhl, hlh),
          lhh: at!(lhh, hll),
          hll: at!(hll, lhh),
          hlh: at!(hlh, lhl),
          hhl: at!(hhl, llh),
          hhh: at!(hhh, lll),
        };
    }
  }

  /// Find a voxel inside this tree.
  /// If it doesn't exist, it will be created as empty.
  #[inline(never)]
  pub fn get_mut_or_create<'a>(&'a mut self, voxel: &bounds::T) -> &'a mut Inner<Voxel> {
    self.grow_to_hold(voxel);

    let mut traversal = traversal::to_voxel_mut(self, voxel);
    let mut tree = &mut self.contents;
    loop {
      let old_tree = tree;
      match traversal.next(old_tree) {
        traversal::Step::Step(new_tree) => {
          tree = new_tree.force_branches();
        },
        traversal::Step::Last(branch) => {
          return branch
        },
      }
    }
  }

  /// Find a voxel inside this tree.
  pub fn get<'a>(&'a self, voxel: &bounds::T) -> Option<&'a Voxel> {
    if !self.contains_bounds(voxel) {
      return None
    }

    match traversal::to_voxel(self, voxel).last(&self.contents) {
      None => None,
      Some(branch) => branch.voxel(),
    }
  }

  #[allow(missing_docs)]
  pub fn get_pointer<'a>(&'a self, voxel: &bounds::T) -> Option<&'a Inner<Voxel>> {
    if !self.contains_bounds(voxel) {
      return None
    }

    traversal::to_voxel(self, voxel).last(&self.contents)
  }

  /// Find a voxel inside this tree.
  pub fn get_mut<'a>(&'a mut self, voxel: &bounds::T) -> Option<&'a mut Voxel> {
    if !self.contains_bounds(voxel) {
      return None
    }

    match traversal::to_voxel_mut(self, voxel).last(&mut self.contents) {
      None => None,
      Some(branch) => branch.voxel_mut(),
    }
  }

  /// Find a voxel inside this tree.
  pub fn get_mut_pointer<'a>(&'a mut self, voxel: &bounds::T) -> Option<&'a mut Inner<Voxel>> {
    if !self.contains_bounds(voxel) {
      return None
    }

    traversal::to_voxel_mut(self, voxel).last(&mut self.contents)
  }

  /// Cast a ray through the contents of this tree.
  pub fn cast_ray<'a, Act, R>(
    &'a self,
    ray: &Ray3<f32>,
    act: &mut Act,
  ) -> Option<R>
    where
      // TODO: Does this *have* to be callback-based?
      Act: FnMut(bounds::T, &'a Voxel) -> Option<R>
  {
    let coords = [
      if ray.origin.x >= 0.0 {1} else {0},
      if ray.origin.y >= 0.0 {1} else {0},
      if ray.origin.z >= 0.0 {1} else {0},
    ];
    // NB: The children are half the size of the tree itself,
    // but tree.lg_size=0 means it extends tree.lg_size=0 in *each direction*,
    // so the "actual" size of the tree as a voxel would be tree.lg_size+1.
    let child_lg_size = self.lg_size as i16;
    let mut make_bounds = |coords: [usize; 3]| {
      bounds::T {
        x: coords[0] as i32 - 1,
        y: coords[1] as i32 - 1,
        z: coords[2] as i32 - 1,
        lg_size: child_lg_size,
      }
    };
    match raycast::cast_ray_branches(
      &self.contents,
      ray,
      None,
      coords,
      &mut make_bounds,
      act,
    ) {
      Ok(r) => Some(r),
      Err(_) => None,
    }
  }

  /// Apply a voxel brush to the contents of this tree.
  pub fn brush<Material, Mosaic, Generate, OnVoxelUpdate>(
    &mut self,
    brush: &mut brush::T<Mosaic>,
    generate: &mut Generate,
    on_voxel_update: &mut OnVoxelUpdate,
  ) where
    Mosaic: mosaic::T<Material>,
    Voxel: ::T<Material>,
    Generate: FnMut(&::bounds::T) -> Option<Voxel>,
    OnVoxelUpdate: FnMut(&Voxel, &::bounds::T),
  {
    macro_rules! recurse(($branch: ident, $x: expr, $y: expr, $z: expr) => {{
      self.contents.$branch.brush(
        &bounds::new($x, $y, $z, self.lg_size as i16),
        brush,
        generate,
        on_voxel_update
      );
    }});
    recurse!(lll, -1, -1, -1);
    recurse!(llh, -1, -1,  0);
    recurse!(lhl, -1,  0, -1);
    recurse!(lhh, -1,  0,  0);
    recurse!(hll,  0, -1, -1);
    recurse!(hlh,  0, -1,  0);
    recurse!(hhl,  0,  0, -1);
    recurse!(hhh,  0,  0,  0);
  }
}

#[cfg(test)]
mod tests {
  extern crate test;

  use std;
  use cgmath::{Ray3, Vector3, Point3};

  use super::{T, Branches, Inner};
  use bounds;
  use brush;
  use field;
  use mosaic;

  #[derive(Debug)]
  struct EraseAll;

  impl field::T for EraseAll {
    fn density(&mut self, _: &Point3<f32>) -> f32 {
      1.0
    }

    fn normal(&mut self, _: &Point3<f32>) -> Vector3<f32> {
      Vector3::new(0.0, 0.0, 0.0)
    }
  }

  impl mosaic::T<()> for EraseAll {
    fn material(&mut self, _: &Point3<f32>) -> Option<()> {
      None
    }
  }

  impl ::T<()> for i32 {
    fn brush<Mosaic>(
      this: &mut Self,
      _: &bounds::T,
      _: &mut brush::T<Mosaic>,
    ) where Mosaic: mosaic::T<()>
    {
      *this = 999;
    }
  }

  #[test]
  fn simple_lookup() {
    let tree: T<i32> =
      T {
        lg_size: 0,
        contents: Branches {
          data: None,
          lll: Inner::leaf(Some(0)),
          llh: Inner::leaf(Some(1)),
          lhl: Inner::leaf(Some(2)),
          lhh: Inner::leaf(Some(3)),
          hll: Inner::leaf(Some(4)),
          hlh: Inner::leaf(Some(5)),
          hhl: Inner::leaf(Some(6)),
          hhh: Inner::leaf(Some(7)),
        },
      };

    assert_eq!(tree.get(&bounds::new(-1, -1, -1, 0)), Some(&0));
    assert_eq!(tree.get(&bounds::new(-1, -1,  0, 0)), Some(&1));
    assert_eq!(tree.get(&bounds::new(-1,  0, -1, 0)), Some(&2));
    assert_eq!(tree.get(&bounds::new(-1,  0,  0, 0)), Some(&3));
    assert_eq!(tree.get(&bounds::new( 0, -1, -1, 0)), Some(&4));
    assert_eq!(tree.get(&bounds::new( 0, -1,  0, 0)), Some(&5));
    assert_eq!(tree.get(&bounds::new( 0,  0, -1, 0)), Some(&6));
    assert_eq!(tree.get(&bounds::new( 0,  0,  0, 0)), Some(&7));
  }

  #[test]
  fn insert_and_lookup() {
    let mut tree: T<i32> = super::new();
    *tree.get_mut_or_create(&bounds::new(1, 1, 1, 0)) = Inner::leaf(Some(1));
    *tree.get_mut_or_create(&bounds::new(8, -8, 4, 0)) = Inner::leaf(Some(2));
    *tree.get_mut_or_create(&bounds::new(2, 0, 4, 4)) = Inner::leaf(Some(3));
    *tree.get_mut_or_create(&bounds::new(9, 0, 16, 2)) = Inner::leaf(Some(4));
    *tree.get_mut_or_create(&bounds::new(9, 0, 16, 2)) = Inner::leaf(Some(5));

    assert_eq!(tree.get(&bounds::new(1, 1, 1, 0)), Some(&1));
    assert_eq!(tree.get(&bounds::new(8, -8, 4, 0)), Some(&2));
    assert_eq!(tree.get(&bounds::new(9, 0, 16, 2)), Some(&5));

    // Bigger LOD encompassing smaller LODs
    assert_eq!(tree.get(&bounds::new(2, 0, 4, 4)), Some(&3));
  }

  #[test]
  fn wrong_voxel_size_is_not_found() {
    let mut tree: T<i32> = super::new();
    *tree.get_mut_or_create(&bounds::new(4, 4, -4, 1)) = Inner::leaf(Some(1));
    assert_eq!(tree.get(&bounds::new(4, 4, -4, 0)), None);
    assert_eq!(tree.get(&bounds::new(4, 4, -4, 2)), None);
  }

  #[test]
  fn grow_is_transparent() {
    let mut tree: T<i32> = super::new();
    *tree.get_mut_or_create(&bounds::new(1, 1, 1, 0)) = Inner::leaf(Some(1));
    tree.grow_to_hold(&bounds::new(0, 0, 0, 1));
    tree.grow_to_hold(&bounds::new(0, 0, 0, 2));
    tree.grow_to_hold(&bounds::new(-32, 32, -128, 3));

    assert_eq!(tree.get(&bounds::new(1, 1, 1, 0)), Some(&1));
  }

  #[test]
  fn simple_cast_ray() {
    let mut tree: T<i32> = super::new();
    *tree.get_mut_or_create(&bounds::new(1, 1, 1, 0)) = Inner::leaf(Some(1));
    *tree.get_mut_or_create(&bounds::new(4, 4, 4, 0)) = Inner::leaf(Some(2));

    let actual = tree.cast_ray(
      &Ray3::new(Point3::new(4.5, 3.0, 4.5), Vector3::new(0.1, 0.8, 0.1)),
      // Return the first voxel we hit.
      &mut |bounds, v| Some((bounds, v)),
    );

    assert_eq!(actual, Some((bounds::new(4, 4, 4, 0), &2)));
  }

  #[test]
  fn simple_remove() {
    let mut tree: T<i32> = super::new();
    *tree.get_mut_or_create(&bounds::new(9, -1, 3, 0)) = Inner::leaf(Some(1));

    tree.brush(
      &mut brush::T {
        mosaic: EraseAll,
        bounds:
          brush::Bounds::new(
            Point3::new(9, -1, 3),
            Point3::new(10, 0, 4),
          ),
        min_lg_size: 0,
      },
      &mut |_| None,
      &mut |_, _| {},
    );

    assert_eq!(tree.get(&bounds::new(9, -1, 3, 0)), Some(&999));
  }

  #[bench]
  fn simple_inserts(bencher: &mut test::Bencher) {
    bencher.iter(|| {
      let mut tree: T<i32> = super::new();
      tree.grow_to_hold(&bounds::new(0, 0, 0, 30));
      for i in 0..1000 {
        *tree.get_mut_or_create(&bounds::new(i, i, i, 0)) = Inner::leaf(Some(i));
      }
      test::black_box(tree);
    });
  }

  #[bench]
  fn simple_inserts_hashmap(bencher: &mut test::Bencher) {
    bencher.iter(|| {
      let mut tree = std::collections::HashMap::new();
      for i in 0..1000 {
        tree.insert(bounds::new(i, i, i, 0), Some(i));
      }
      test::black_box(tree);
    });
  }

  #[bench]
  fn bench_cast_ray(bencher: &mut test::Bencher) {
    let mut tree: T<i32> = super::new();
    tree.grow_to_hold(&bounds::new(0, 0, 0, 30));
    *tree.get_mut_or_create(&bounds::new(1, 1, 1, 0)) = Inner::leaf(Some(1));
    *tree.get_mut_or_create(&bounds::new(4, 4, 4, 0)) = Inner::leaf(Some(2));

    bencher.iter(|| {
      let r = tree.cast_ray(
        &Ray3::new(Point3::new(4.5, 3.0, 4.5), Vector3::new(0.1, 0.8, 0.1)),
        // Return the first voxel we hit.
        &mut |bounds, v| Some((bounds, v)),
      );
      test::black_box(r);
    });
  }
}
