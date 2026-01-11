use std::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Mul};
use std::sync::atomic::AtomicU64;

use crate::sim::common::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GripStatus {
    Active,
    Blocked,
    Inactive,
}

/// Layer mask of a block, represented using 12 bits: `0www_0zzz_0yyy_0xxx`
/// (MSb..LSb)
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackedLayers([u8; 2]);

impl PackedLayers {
    pub const EMPTY: Self = Self::from_u16(0);
    pub const CORE_3D: Self = Self::from_u16(0b_0111_0010_0010_0010);
    pub const CORE_4D: Self = Self::from_u16(0b_0010_0010_0010_0010);
    pub const ALL: Self = Self::from_u16(0b_0111_0111_0111_0111);

    #[inline]
    pub(crate) const fn to_u16(self) -> u16 {
        u16::from_ne_bytes(self.0)
    }
    #[inline]
    pub(crate) const fn from_u16(i: u16) -> Self {
        // debug_assert!(i != 0);
        debug_assert!(i & 0x8888 == 0);
        // debug_assert!(i & 0x0007 != 0x0005);
        // debug_assert!(i & 0x0070 != 0x0050);
        // debug_assert!(i & 0x0700 != 0x0500);
        // debug_assert!(i & 0x7000 != 0x5000);
        Self(i.to_ne_bytes())
    }

    #[must_use]
    pub const fn restrict_to_active_grip(self, g: GripId) -> Option<Self> {
        Self::from_u16(self.to_u16() & !inactive_grip_mask(g)).if_nonempty_on_axis(g.axis())
    }
    #[must_use]
    pub const fn restrict_to_inactive_grip(self, g: GripId) -> Option<Self> {
        Self::from_u16(self.to_u16() & !active_grip_mask(g)).if_nonempty_on_axis(g.axis())
    }
    #[must_use]
    pub const fn expand_to_active_grip(self, g: GripId) -> Self {
        Self::from_u16(
            self.to_u16()
                | match self.grip_status(g.opposite()) {
                    GripStatus::Active => inactive_grip_mask(g.opposite()),
                    _ => active_grip_mask(g),
                },
        )
    }

    #[inline]
    const fn if_nonempty_on_axis(self, axis: usize) -> Option<Self> {
        if self.is_empty_on_axis(axis) {
            None
        } else {
            Some(self)
        }
    }
    #[inline]
    pub const fn is_empty_on_any_axis(self) -> bool {
        let bits = self.to_u16();
        (bits | (bits >> 1) | (bits >> 2)) & 0b_0001_0001_0001_0001 != 0b_0001_0001_0001_0001
    }
    #[inline]
    pub const fn is_fully_blocked_on_axis(self, axis: usize) -> bool {
        self.bits_for_axis(axis) == 0b111
    }
    #[inline]
    pub fn has_middle_slice_on_axis(self, axis: usize) -> bool {
        (self.to_u16() >> (axis * 4) & 0b010) != 0
    }

    #[inline]
    pub const fn bits_for_axis(self, axis: usize) -> u8 {
        debug_assert!(axis < 4, "axis out of range");
        ((self.to_u16() >> (axis * 4)) & 0b111) as u8
    }
    #[inline]
    const fn is_empty_on_axis(self, axis: usize) -> bool {
        self.bits_for_axis(axis) == 0
    }
    #[inline]
    pub const fn is_axis_blocked(self, axis: usize) -> bool {
        self.bits_for_axis(axis) & 0b101 != 0
    }

    #[inline]
    const fn bits_for_grip(self, g: GripId) -> u8 {
        let bits = self.bits_for_axis(g.axis());
        if g.id() & 1 == 0 { bits } else { rev3(bits) }
    }

    /// Returns a bitmask of axes along which at least one grip is blocked.
    #[inline]
    pub const fn blocked_axes_mask(self) -> u16 {
        let bits = self.to_u16();
        (bits | (bits >> 2)) & 0b_0001_0001_0001_0001
    }
    #[inline]
    pub fn indistinguishable_subgroup(self) -> Option<&'static [ElemId]> {
        let blocked_axes_mask = self.blocked_axes_mask();
        let num_blocked_axes = blocked_axes_mask.count_ones();
        match num_blocked_axes {
            1 => Some(match blocked_axes_mask {
                0b_0000_0000_0000_0001 => X_STABILIZER.as_slice(),
                0b_0000_0000_0001_0000 => Y_STABILIZER.as_slice(),
                0b_0000_0001_0000_0000 => Z_STABILIZER.as_slice(),
                0b_0001_0000_0000_0000 => W_STABILIZER.as_slice(),
                _ => return None,
            }),
            2 => Some(match blocked_axes_mask {
                0b_0000_0000_0001_0001 => XY_STABILIZER.as_slice(),
                0b_0000_0001_0000_0001 => XZ_STABILIZER.as_slice(),
                0b_0001_0000_0000_0001 => XW_STABILIZER.as_slice(),
                0b_0000_0001_0001_0000 => YZ_STABILIZER.as_slice(),
                0b_0001_0000_0001_0000 => YW_STABILIZER.as_slice(),
                0b_0001_0001_0000_0000 => ZW_STABILIZER.as_slice(),
                _ => return None,
            }),
            _ => None,
        }
    }
    /// Returns `[inside, outside]`
    #[must_use]
    pub fn split(self, g: GripId) -> [Option<Self>; 2] {
        [
            self.restrict_to_active_grip(g),
            self.restrict_to_inactive_grip(g),
        ]
    }

    pub fn is_subset_of(self, other: Self) -> bool {
        self.to_u16() & !other.to_u16() == 0
    }

    /// Merges the blocks if possible. Returns the merged block and the axis
    /// along which they were merged.
    // cargo asm --rust --lib robodoan::sim::blockbuilding::layer_mask::PackedLayers::try_merge_with_no_ret_axis
    #[must_use]
    #[inline(never)]
    #[cfg(false)]
    pub fn try_merge_with_no_ret_axis(self, other: Self) -> Option<Self> {
        let lhs = self.to_u16();
        let rhs = other.to_u16();
        let layer_difference = lhs ^ rhs;
        // if layer_difference.count_ones() > 3 {
        //     return None; // multiple axes are guaranteed to have different layers
        // }

        // {
        //     let different_axes = (layer_difference & 0x0007 != 0) as u32
        //         + (layer_difference & 0x0070 != 0) as u32
        //         + (layer_difference & 0x0700 != 0) as u32
        //         + (layer_difference & 0x7000 != 0) as u32;
        //     // assert_ne!(different_axes, 0);
        //     if different_axes != 1 {
        //         COUNTER_A.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //         return None;
        //     } else {
        //         COUNTER_B.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //     }
        //     // if different_axes != 1 {
        //     //     if layer_difference.count_ones() > 3 {
        //     //         COUNTER_A.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //     //     } else {
        //     //         COUNTER_B.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //     //     }
        //     // }
        // }

        // let merge_axis = layer_difference.trailing_zeros() as usize / 4;
        // debug_assert!(merge_axis < 4, "same block!"); // otherwise they'd be the same blocks

        let mask = {
            let mut mask = layer_difference;
            mask |= mask >> 1;
            mask |= mask << 1;
            // at this point, the middle bit is 1 iff any of the bits are 1
            mask &= 0x7777; // to avoid spilling into other axes
            mask |= mask >> 1;
            mask |= mask << 1;
            // at this point, mask will be 0b*111 at each axis where there is a difference and 0b*000 otherwise, where * is undefined
            mask
        };
        // let left_mask = {
        //     let mut left_mask = mask;
        //     left_mask |= left_mask << 4;
        //     left_mask |= left_mask << 8;
        //     // and now left_mask will select all axes to the left of the difference as well
        //     left_mask
        // };

        // if layer_difference & (left_mask << 4) != 0 {
        // assert_eq!(
        //     layer_difference & (mask << 4 | mask << 8 | mask << 12) != 0,
        //     layer_difference.unbounded_shr((merge_axis as u32 + 1) * 4) != 0
        // );
        if layer_difference & (mask << 4 | mask << 8 | mask << 12) != 0 {
            return None; // multiple axes have different layers
        }

        let or = lhs | rhs;
        // assert_eq!(
        //     or & mask == 0b0101_0101_0101_0101 & mask,
        //     self.bits_for_axis(merge_axis) | other.bits_for_axis(merge_axis) == 0b101
        // );
        // assert_eq!(
        //     or & mask == 0b0101_0101_0101_0101 & mask,
        //     (or ^ 0b0101_0101_0101_0101) & mask == 0
        // );
        if or & mask == 0b0101_0101_0101_0101 & mask {
            return None; // disconnected blocks not allowed (but they could be, if we had slice moves)
        }

        Some(PackedLayers::from_u16(or))

        // let diff0 = layer_difference & 0b0111;
        // // let diff1 = layer_difference & 0b0111_0000;
        // let diff1 = (layer_difference >> 4) & 0b0111;

        // let lhs_shifted = lhs >> (merge_axis * 4);
        // let rhs_shifted = rhs >> (merge_axis * 4);
        // let or = lhs_shifted | rhs_shifted;
        // // if or & 0xFFF0 != 0 {
        // //     return None; // multiple axes have different layers
        // // }
        // // if or & 0b111 == 0b101 {
        // //     return None; // disconnected blocks not allowed (but they could be, if we had slice moves)
        // // }

        // if layer_difference.unbounded_shr((merge_axis as u32 + 1) * 4) != 0 {
        //     return None; // multiple axes have different layers
        // }

        // let axis_mask = 0b111 << (merge_axis * 4);
        // if ((lhs | rhs) & axis_mask & 0b0101_0101_0101_0101) != 0 {
        //     return None; // disconnected blocks not allowed (but they could be, if we had slice moves)
        // }

        // // // let lhs_axis_bits = self.bits_for_axis(merge_axis);
        // // // let rhs_axis_bits = other.bits_for_axis(merge_axis);
        // let lhs_axis_bits = (lhs >> (merge_axis * 4)) & 0b111;
        // let rhs_axis_bits = (rhs >> (merge_axis * 4)) & 0b111;

        // // assert!(lhs_axis_bits & rhs_axis_bits == 0, "blocks shouldn't overlap");
        // // if lhs_axis_bits & rhs_axis_bits != 0 {
        // //     return None; // `self` and `other` overlap
        // // }

        // assert_eq!(or & 0b111 == 0b101, lhs_axis_bits | rhs_axis_bits == 0b101);
        // if lhs_axis_bits | rhs_axis_bits == 0b101 {
        //     return None; // disconnected blocks not allowed (but they could be, if we had slice moves)
        // }

        // Some((self | other, merge_axis as usize))
    }
    #[must_use]
    #[inline(never)]
    pub fn try_merge_with_no_ret_axis(self, other: Self) -> Option<Self> {
        let lhs = self.to_u16();
        let rhs = other.to_u16();
        let layer_difference = lhs ^ rhs;
        let merge_axis = layer_difference.trailing_zeros() as usize / 4;
        debug_assert!(merge_axis < 4);

        // if layer_difference
        //     & match merge_axis {
        //         0 => 0x7770,
        //         1 => 0x7707,
        //         2 => 0x7077,
        //         3 => 0x0777,
        //         _ => unreachable!(),
        //     }
        //     != 0
        if layer_difference & !(0x7 << (merge_axis * 4)) != 0 {
            // TODO: try_merge that assumes this
            // panic!("we should be checking for this already");
            return None; // multiple axes have different layers
        }

        let lhs_axis_bits = self.bits_for_axis(merge_axis);
        let rhs_axis_bits = other.bits_for_axis(merge_axis);

        debug_assert!(
            lhs_axis_bits & rhs_axis_bits == 0,
            "blocks shouldn't overlap"
        );

        if lhs_axis_bits | rhs_axis_bits == 0b101 {
            return None; // disconnected blocks not allowed
        }

        Some(self | other)
    }
    #[must_use]
    #[inline(never)]
    pub fn try_merge_with_along_axis(self, other: Self, merge_axis: usize) -> Option<Self> {
        let lhs = self.to_u16();
        let rhs = other.to_u16();
        let layer_difference = lhs ^ rhs;
        debug_assert!(merge_axis < 4);
        debug_assert_eq!(layer_difference.trailing_zeros() as usize / 4, merge_axis);

        // if layer_difference
        //     & match merge_axis {
        //         0 => 0x7770,
        //         1 => 0x7707,
        //         2 => 0x7077,
        //         3 => 0x0777,
        //         _ => unreachable!(),
        //     }
        //     != 0
        if layer_difference & !(0x7 << (merge_axis * 4)) != 0 {
            return None; // multiple axes have different layers
        }

        let lhs_axis_bits = self.bits_for_axis(merge_axis);
        let rhs_axis_bits = other.bits_for_axis(merge_axis);

        debug_assert!(
            lhs_axis_bits & rhs_axis_bits == 0,
            "blocks shouldn't overlap"
        );

        if lhs_axis_bits | rhs_axis_bits == 0b101 {
            return None; // disconnected blocks not allowed
        }

        Some(self | other)
    }
    #[must_use]
    #[inline(never)]
    pub fn try_merge_with_ret_axis(self, other: Self) -> Option<(Self, usize)> {
        let lhs = self.to_u16();
        let rhs = other.to_u16();
        let layer_difference = lhs ^ rhs;
        Some((
            Self::try_merge_with_no_ret_axis(self, other)?,
            layer_difference.trailing_zeros() as usize / 4,
        ))
    }

    /// Returns the positive grips on the separating axes of the blocks.
    pub fn separating_axes(self, other: Self) -> GripSet {
        let diff = self ^ other;
        let mut ret = GripSet::NONE;
        for g in [R, U, F, O] {
            if diff.bits_for_grip(g) & 0b111 != 0 {
                ret += g;
            }
        }
        ret
    }

    /// Returns the number of active grips the block has.
    // #[inline]
    #[inline(never)]
    pub const fn active_grip_count(self) -> u32 {
        let bits = self.to_u16();
        let pos_bits = bits & 0b_0001_0001_0001_0001;
        let mid_bits = bits & 0b_0010_0010_0010_0010;
        let neg_bits = bits & 0b_0100_0100_0100_0100;
        let pos_grips_active = pos_bits & (!mid_bits >> 1);
        let neg_grips_active = neg_bits & (!mid_bits << 1);
        (pos_grips_active | neg_grips_active).count_ones()
    }

    /// Returns the status of a grip, or `None` if the layer mask is invalid.
    pub const fn grip_status(self, g: GripId) -> GripStatus {
        match self.bits_for_grip(g) {
            0b001 => GripStatus::Active,
            0b011 | 0b111 => GripStatus::Blocked,
            0b110 | 0b100 | 0b010 => GripStatus::Inactive,
            _ => panic!("invalid block"),
        }
    }

    #[cfg(test)]
    pub(crate) fn unpack(self) -> [u8; 4] {
        let bits = self.to_u16();
        [0, 4, 8, 12].map(|i| (bits >> i) as u8 & 0b111)
    }
}

const fn active_grip_mask(g: GripId) -> u16 {
    g.hint_assert_in_bounds();
    0b01 << (g.id() as usize * 2)
}
const fn inactive_grip_mask(g: GripId) -> u16 {
    g.hint_assert_in_bounds();
    (0b0111 << (g.axis() * 4)) ^ active_grip_mask(g)
}

impl fmt::Debug for PackedLayers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:03b}", self.bits_for_axis(3))?;
        write!(f, "_{:03b}", self.bits_for_axis(2))?;
        write!(f, "_{:03b}", self.bits_for_axis(1))?;
        write!(f, "_{:03b}", self.bits_for_axis(0))?;
        assert_eq!(self.to_u16() & !Self::ALL.to_u16(), 0);
        Ok(())
    }
}
impl fmt::Display for PackedLayers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut active = GripSet::NONE;
        let mut blocked = GripSet::NONE;
        let mut inactive = GripSet::NONE;
        for g in HYPERCUBE_GRIPS {
            match self.grip_status(g) {
                GripStatus::Active => active += g,
                GripStatus::Blocked => blocked += g,
                GripStatus::Inactive => inactive += g,
                // None => return write!(f, "bad_mask_{:#b}", self.to_u16()),
            }
        }
        write!(f, "{active}${blocked:#}!{inactive}")
    }
}

impl BitOr for PackedLayers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_u16(self.to_u16() | rhs.to_u16())
    }
}

impl BitAnd for PackedLayers {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_u16(self.to_u16() & rhs.to_u16())
    }
}

impl BitXor for PackedLayers {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_u16(self.to_u16() ^ rhs.to_u16())
    }
}

impl Mul<PackedLayers> for ElemId {
    type Output = PackedLayers;

    #[inline(always)]
    fn mul(self, rhs: PackedLayers) -> Self::Output {
        let inv = self.inv();
        let mut ret = 0_u16;
        for g in [R, U, F, O] {
            ret |= (rhs.bits_for_grip(inv * g) as u16) << (g.axis() * 4);
        }
        PackedLayers::from_u16(ret)
    }
}

/// Reverses the lowest 3 bits of a byte.
const fn rev3(x: u8) -> u8 {
    x.reverse_bits() >> 5
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_transform_layer_mask() {
        let layers = PackedLayers::from_u16(0b_0111_0100_0010_0110);
        assert_eq!(IDENT * layers, layers);
        for &t1 in &*HYPERCUBE_ROTATIONS {
            for &t2 in &*HYPERCUBE_ROTATIONS {
                assert_eq!(t1 * (t2 * layers), (t1 * t2) * layers);
            }
        }
    }

    proptest! {
        #[test]
        fn proptest_merge_layers(xs in prop::array::uniform8(0..6_usize)) {
            test_merge_layers(xs)
        }
    }

    fn test_merge_layers(xs: [usize; 8]) {
        let axis_options = [0b001, 0b010, 0b100, 0b011, 0b110, 0b111];
        let l1 = PackedLayers::from_u16(
            axis_options[xs[0]]
                | (axis_options[xs[0]] << 4)
                | (axis_options[xs[1]] << 8)
                | (axis_options[xs[2]] << 12),
        );
        let l2 = PackedLayers::from_u16(
            axis_options[xs[4]]
                | (axis_options[xs[5]] << 4)
                | (axis_options[xs[6]] << 8)
                | (axis_options[xs[7]] << 12),
        );

        if l1 == l2 {
            return;
        }

        let merge_axis = (0..4)
            .filter(|&axis| l1.bits_for_axis(axis) != l2.bits_for_axis(axis))
            .exactly_one()
            .ok();

        let merge_grip = merge_axis.and_then(|axis| {
            GripId::pair_on_axis(axis).into_iter().find(|&g| {
                (l1.bits_for_grip(g) == 0b001 && l2.bits_for_grip(g) & 0b011 == 0b010)
                    || (l2.bits_for_grip(g) == 0b001 && l1.bits_for_grip(g) & 0b011 == 0b010)
            })
        });

        let expected = merge_grip.map(|g| (l1 | l2, g.axis()));
        let actual = l1.try_merge_with_ret_axis(l2);

        assert_eq!(expected, actual);
    }
}
