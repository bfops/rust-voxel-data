#![allow(missing_docs)]

fn to_voxel_mask<Voxel>(tree: &::tree::T<Voxel>, bounds: &::bounds::T) -> i32 {
  // When we compare the voxel position to octree bounds to choose subtrees
  // for insertion, we'll be comparing voxel position to values of 2^n and
  // -2^n, so we can just use the position bits to branch directly.
  // This actually works for negative values too, without much wrestling:
  // we need to branch on the sign bit up front, but after that, two's
  // complement magic means the branching on bits works regardless of sign.

  let mut mask = (1 << tree.lg_size) >> 1;

  // Shift everything by the voxel's lg_size, so we can compare the mask to 0
  // to know whether we're done.
  if bounds.lg_size >= 0 {
    mask = mask >> bounds.lg_size;
  } else {
    // TODO: Check for overflow.
    mask = mask << -bounds.lg_size;
  }

  mask
}

pub fn to_voxel_mut<Voxel>(tree: &::tree::T<Voxel>, bounds: &::bounds::T) -> ToVoxelMut {
  ToVoxelMut {
    target: *bounds,
    mask: to_voxel_mask(tree, bounds),
    first: true,
  }
}

pub fn to_voxel<Voxel>(tree: &::tree::T<Voxel>, bounds: &::bounds::T) -> ToVoxel {
  ToVoxel {
    target: *bounds,
    mask: to_voxel_mask(tree, bounds),
    first: true,
  }
}

pub enum Step<T> {
  Step(T),
  Last(T),
}

pub struct ToVoxelMut {
  target: ::bounds::T,
  mask: i32,
  first: bool,
}

impl ToVoxelMut {
  fn select(&self, x: i32) -> usize {
    if self.first {
      (x >= 0) as usize
    } else {
      ((x & self.mask) != 0) as usize
    }
  }

  pub fn next<'a, Voxel>(
    &mut self,
    tree: &'a mut ::tree::Branches<Voxel>,
  ) -> Step<&'a mut ::tree::Node<Voxel>> {
    let tree_tmp = tree;
    let branch =
      &mut tree_tmp.as_array_mut()
        [self.select(self.target.x)]
        [self.select(self.target.y)]
        [self.select(self.target.z)]
      ;

    if self.first {
      self.first = false;
    } else {
      self.mask = self.mask >> 1;
    }

    // We've reached the voxel.
    if self.mask == 0 {
      Step::Last(branch)
    } else {
      Step::Step(branch)
    }
  }

  pub fn last<'a, Voxel>(
    &mut self,
    mut tree: &'a mut ::tree::Branches<Voxel>,
  ) -> Option<&'a mut ::tree::Node<Voxel>> {
    loop {
      let old_tree = tree;
      match self.next(old_tree) {
        Step::Last(x) => return Some(x),
        Step::Step(node) => {
          use ::tree::Inner::*;
          match node.next {
            Empty => return None,
            Branches(ref mut new_tree) => {
              tree = new_tree;
            }
          }
        },
      }
    }
  }
}

pub struct ToVoxel {
  target: ::bounds::T,
  mask: i32,
  first: bool,
}

impl ToVoxel {
  fn select(&self, x: i32) -> usize {
    if self.first {
      (x >= 0) as usize
    } else {
      ((x & self.mask) != 0) as usize
    }
  }

  pub fn next<'a, Voxel>(
    &mut self,
    tree: &'a ::tree::Branches<Voxel>,
  ) -> Step<&'a ::tree::Node<Voxel>> {
    let tree_tmp = tree;
    let branch =
      &tree_tmp.as_array()
        [self.select(self.target.x)]
        [self.select(self.target.y)]
        [self.select(self.target.z)]
      ;

    if self.first {
      self.first = false;
    } else {
      self.mask = self.mask >> 1;
    }

    // We've reached the voxel.
    if self.mask == 0 {
      Step::Last(branch)
    } else {
      Step::Step(branch)
    }
  }

  pub fn last<'a, Voxel>(
    &mut self,
    mut tree: &'a ::tree::Branches<Voxel>,
  ) -> Option<&'a ::tree::Node<Voxel>> {
    loop {
      match self.next(tree) {
        Step::Last(x) => return Some(x),
        Step::Step(node) => {
          use ::tree::Inner::*;
          match node.next {
            Empty => return None,
            ::tree::Inner::Branches(ref new_tree) => {
              tree = new_tree;
            },
          }
        },
      }
    }
  }
}
