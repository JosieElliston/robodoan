use std::fmt;
use std::ops::Mul;

use super::elements::*;
use super::group;
use super::space::*;

// #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct GripId(u8);

// pub const R: GripId = GripId::new(0);
// pub const L: GripId = GripId::new(1);
// pub const U: GripId = GripId::new(2);
// pub const D: GripId = GripId::new(3);
// pub const F: GripId = GripId::new(4);
// pub const B: GripId = GripId::new(5);
// pub const O: GripId = GripId::new(6);
// pub const I: GripId = GripId::new(7);

// pub const HYPERCUBE_GRIPS: [GripId; 8] = [R, L, U, D, F, B, O, I];
// pub const CUBE_GRIPS: [GripId; 6] = [R, L, U, D, F, B];

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum GripId {
    #[default]
    R = 0,
    L = 1,
    U = 2,
    D = 3,
    F = 4,
    B = 5,
    O = 6,
    I = 7,
}

pub use GripId::{B, D, F, I, L, O, R, U};

pub const HYPERCUBE_GRIPS: [GripId; 8] = [R, L, U, D, F, B, O, I];
pub const CUBE_GRIPS: [GripId; 6] = [R, L, U, D, F, B];

impl fmt::Display for GripId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.char().to_ascii_lowercase())
        } else {
            write!(f, "{}", self.char())
        }
    }
}

impl GripId {
    pub const fn id(self) -> u8 {
        self as u8
    }
    pub const fn axis(self) -> usize {
        self.id() as usize >> 1
    }
    pub const fn signum(self) -> i8 {
        if self.id() & 1 == 0 { 1 } else { -1 }
    }

    #[inline]
    pub const fn pair_on_axis(axis: usize) -> [Self; 2] {
        assert!(axis < 4, "axis out of range");
        let g1 = Self::new((axis as u8) << 1);
        [g1, g1.opposite()]
    }

    /// Constructs a grip from an ID.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of range (must be strictly less 8).
    pub const fn new(id: u8) -> Self {
        Self::try_new(id).expect("grip ID out of range")
    }
    /// Constructs a grip from an ID, or returns `None` if `id` is out of range
    /// (must be strictly less than 8).
    pub const fn try_new(id: u8) -> Option<Self> {
        // if id < 8 { Some(Self(id)) } else { None }
        match id {
            0 => Some(R),
            1 => Some(L),
            2 => Some(U),
            3 => Some(D),
            4 => Some(F),
            5 => Some(B),
            6 => Some(O),
            7 => Some(I),
            _ => None,
        }
    }

    // TODO: remove
    /// Hint to the compiler that the grip ID is within bounds.
    #[inline]
    pub const fn hint_assert_in_bounds(self) -> Self {
        // SAFETY: `GripId` is only ever constructed using `GripId::try_new()`,
        // which returns `None` if the ID is greater than or equal to 8.
        unsafe { std::hint::assert_unchecked(self.id() < 8) };
        self
    }

    pub const fn opposite(self) -> Self {
        // Self(self.id() ^ 1)
        Self::new(self.id() ^ 1)
    }

    pub const fn char(self) -> char {
        b"RLUDFBOI"[self.id() as usize] as char
    }

    pub fn vec(self) -> Vec4 {
        let mut ret = ZERO;
        ret[self.axis()] = self.signum();
        ret
    }

    pub fn transforms(self, subgroup: &[ElemId]) -> Vec<ElemId> {
        group::stabilizer(subgroup.iter().copied(), self.vec())
            .filter(|&e| e != IDENT)
            .collect()
    }

    pub fn can_transform_grip_to_grip(self, start: GripId, end: GripId) -> bool {
        self.axis() != start.axis() && self.axis() != end.axis()
    }
}

impl Mul<GripId> for ElemId {
    type Output = GripId;

    #[inline]
    fn mul(self, rhs: GripId) -> Self::Output {
        self.hint_assert_in_bounds();
        rhs.hint_assert_in_bounds();
        group::CHIRAL_BC4.mul_elem_grip[self.id() as usize][rhs.id() as usize]
            .hint_assert_in_bounds()
    }
}
