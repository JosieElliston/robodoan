use cgmath::{InnerSpace, vec4};
use itertools::Itertools;

use crate::{
    ELEM_COUNT, ElemId, GripId, GripSet, HYPERCUBE_GRIPS, HYPERCUBE_ROTATIONS, IDENT, Twist, Vec4,
};

/// Piece indices belonging to each grip's cell, ordered by `(z, y, x)` within
/// the cell (skipping the cell's own center).
pub const INDICES_FOR_GRIP: [[u8; 26]; 8] = [
    indices_for_grip(GripId::new(0)),
    indices_for_grip(GripId::new(1)),
    indices_for_grip(GripId::new(2)),
    indices_for_grip(GripId::new(3)),
    indices_for_grip(GripId::new(4)),
    indices_for_grip(GripId::new(5)),
    indices_for_grip(GripId::new(6)),
    indices_for_grip(GripId::new(7)),
];

#[static_init::dynamic]
static MUL_ELEM_INDEX: [[u8; 72]; ELEM_COUNT] = gen_mul_elem_index_table();

/// Bitmask over the 72 piece indices, selecting the 26 pieces of each grip's
/// cell.
pub const CELL_PIECE_MASK: [u128; 8] = {
    let mut ret = [0; 8];
    let mut g = 0;
    while g < 8 {
        let mut i = 0;
        while i < 26 {
            ret[g] |= 1 << INDICES_FOR_GRIP[g][i];
            i += 1;
        }
        g += 1;
    }
    ret
};

/// Indices *within* [`INDICES_FOR_GRIP`] of the pieces of each type in a cell.
///
/// A cell of the 3^4 is a 3x3x3 cube whose 26 non-center pieces are 6 ridges
/// (2c), 12 edges (3c), and 8 corners (4c).
pub const CELL_RIDGES: [usize; 6] = [4, 10, 12, 13, 15, 21];
/// See [`CELL_RIDGES`].
pub const CELL_EDGES: [usize; 12] = [1, 3, 5, 7, 9, 11, 14, 16, 18, 20, 22, 24];
/// See [`CELL_RIDGES`].
pub const CELL_CORNERS: [usize; 8] = [0, 2, 6, 8, 17, 19, 23, 25];

/// Grips touched by the piece whose home is each of the 72 piece locations.
///
/// This is exactly the set of stickers the piece shows, so it determines when a
/// piece counts as solved: an attitude is "solved" at a location iff it fixes
/// every one of these grips.
#[static_init::dynamic]
pub static PIECE_GRIPS: [GripSet; 72] = piece_locations()
    .map(|v| {
        HYPERCUBE_GRIPS
            .into_iter()
            .filter(|g| v[g.axis()] == g.signum())
            .collect()
    })
    .collect_array()
    .unwrap();

/// For each piece location, maps each attitude to the least attitude that looks
/// identical to it.
///
/// A piece at location `i` shows, at facet `g`, whichever of its stickers
/// started out at facet `attitude.inv() * g`. So two attitudes are
/// indistinguishable exactly when they agree on `attitude.inv() * g` for every
/// facet the location has.
///
/// Only ridges (2c pieces) have a non-trivial class here; they show two
/// stickers, and the rotations of the two axes they *don't* touch are
/// invisible.
#[static_init::dynamic]
static INDISTINGUISHABLE_ATTITUDE_CLASS: [[u8; ELEM_COUNT]; 72] = std::array::from_fn(|i| {
    let grips = PIECE_GRIPS[i];
    let mut representative_of_visible_stickers = std::collections::HashMap::new();
    std::array::from_fn(|id| {
        let inv = ElemId::new(id as u8).inv();
        let visible_stickers = grips.iter().map(|g| (inv * g).id()).collect_vec();
        *representative_of_visible_stickers
            .entry(visible_stickers)
            .or_insert(id as u8)
    })
});

/// Returns the least attitude that is indistinguishable from `attitude` for a
/// piece at location `i`.
///
/// Only ridges are ever changed; every other piece shows enough stickers to pin
/// its attitude down exactly.
pub fn canonical_attitude(i: usize, attitude: ElemId) -> ElemId {
    ElemId::new(INDISTINGUISHABLE_ATTITUDE_CLASS[i][attitude.id() as usize])
}

/// Returns the location vector of each of the 72 pieces, in index order.
fn piece_locations() -> impl Iterator<Item = Vec4> {
    itertools::iproduct!(-1..=1, -1..=1, -1..=1, -1..=1)
        .map(|(w, z, y, x)| vec4(x, y, z, w))
        .filter(|&v| vec4_to_index(v).is_some())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PuzzleState {
    /// Piece attitudes, excluding 1 core and 8 centers.
    ///
    /// Index corresponds to *current* piece location, not original piece
    /// location. Pieces are permuted during twists.
    piece_attitudes: [ElemId; 72],
}
impl Default for PuzzleState {
    fn default() -> Self {
        Self {
            piece_attitudes: [IDENT; 72],
        }
    }
}
impl PuzzleState {
    pub fn do_twist(&mut self, twist: Twist) {
        let mut ret = *self;

        // permute & reorient
        for old_index in INDICES_FOR_GRIP[twist.grip.id() as usize] {
            let new_index = MUL_ELEM_INDEX[twist.transform.id() as usize][old_index as usize];
            ret.piece_attitudes[new_index as usize] =
                twist.transform * self.piece_attitudes[old_index as usize];
        }

        *self = ret;
    }
    pub fn do_twists(&mut self, twists: &[Twist]) {
        for &twist in twists {
            self.do_twist(twist);
        }
    }
    pub fn unoriented_pieces(&self, last_layer: GripId) -> [usize; 3] {
        let piece_indices = INDICES_FOR_GRIP[last_layer.id() as usize];
        let is_piece_unoriented = |&i: &usize| {
            self.piece_attitudes[piece_indices[i] as usize] * last_layer.vec() != last_layer.vec()
        };

        let ridges = [4, 10, 12, 13, 15, 21]
            .into_iter()
            .filter(is_piece_unoriented)
            .count();
        let edges = [1, 3, 5, 7, 9, 11, 14, 16, 18, 20, 22, 24]
            .into_iter()
            .filter(is_piece_unoriented)
            .count();
        let corners = [0, 2, 6, 8, 17, 19, 23, 25]
            .into_iter()
            .filter(is_piece_unoriented)
            .count();

        [ridges, edges, corners]
    }

    pub fn piece_attitudes(&self) -> &[ElemId; 72] {
        &self.piece_attitudes
    }

    /// Returns the location that the piece currently at `i` belongs in.
    ///
    /// Attitudes accumulate from a piece's home, so the piece at `i` came from
    /// `attitude.inv() * v`, where `v` is the location vector of `i`.
    pub fn piece_home(&self, i: usize) -> usize {
        MUL_ELEM_INDEX[self.piece_attitudes[i].inv().id() as usize][i] as usize
    }

    /// Rotates the whole puzzle, leaving it just as solved (or unsolved) as it
    /// was.
    ///
    /// This is the state-space counterpart of [`ElemId::transform`] on a twist:
    /// reorienting and then twisting `t` is the same as twisting
    /// `elem.transform(t)` and then reorienting.
    #[must_use]
    pub fn reorient(&self, elem: ElemId) -> Self {
        let mut ret = *self;
        let inv = elem.inv();
        for i in 0..72 {
            let new_index = MUL_ELEM_INDEX[elem.id() as usize][i] as usize;
            ret.piece_attitudes[new_index] = elem * self.piece_attitudes[i] * inv;
        }
        ret
    }

    /// Returns whether the piece at location `i` is solved, accounting for
    /// indistinguishable attitudes (see [`PIECE_GRIPS`]).
    pub fn is_piece_solved(&self, i: usize) -> bool {
        let attitude = self.piece_attitudes[i];
        PIECE_GRIPS[i].iter().all(|g| attitude * g == g)
    }

    /// Returns whether the piece at location `i` belongs in `grip`'s cell.
    ///
    /// The piece's home stickers are `attitude.inv() * PIECE_GRIPS[i]`, so it
    /// belongs to `grip`'s cell exactly when that set contains `grip`.
    pub fn piece_belongs_to_cell(&self, i: usize, grip: GripId) -> bool {
        PIECE_GRIPS[i].contains(self.piece_attitudes[i] * grip)
    }

    pub fn is_solved(&self) -> bool {
        (0..72).all(|i| self.is_piece_solved(i))
    }

    /// Returns whether every piece outside of `last_cell`'s cell is solved,
    /// i.e. whether F2L is intact.
    ///
    /// Note that this implies every piece *of* `last_cell`'s cell is somewhere
    /// in that cell, since the other 46 locations are all accounted for.
    pub fn is_f2l_solved(&self, last_cell: GripId) -> bool {
        let last_cell_mask = CELL_PIECE_MASK[last_cell.id() as usize];
        (0..72).all(|i| last_cell_mask >> i & 1 != 0 || self.is_piece_solved(i))
    }

    /// Returns the number of pieces outside of `last_cell`'s cell that are not
    /// solved.
    pub fn unsolved_pieces_outside_cell(&self, last_cell: GripId) -> usize {
        let last_cell_mask = CELL_PIECE_MASK[last_cell.id() as usize];
        (0..72)
            .filter(|&i| last_cell_mask >> i & 1 == 0 && !self.is_piece_solved(i))
            .count()
    }

    /// Returns the attitudes of the 26 pieces in `last_cell`'s cell, ordered by
    /// [`INDICES_FOR_GRIP`].
    ///
    /// This is the complete last-cell state, and is only meaningful when F2L is
    /// intact.
    pub fn cell_attitudes(&self, last_cell: GripId) -> [ElemId; 26] {
        INDICES_FOR_GRIP[last_cell.id() as usize].map(|i| self.piece_attitudes[i as usize])
    }

    /// Returns a key identifying everything about this state *except* which
    /// last-cell piece is where.
    ///
    /// Two states share a key iff they agree on the placement of all 46 pieces
    /// that don't belong to `last_cell`'s cell. This is the equivalence used to
    /// find F2L-preserving algorithms by meet-in-the-middle: if `a` and `b`
    /// reach states with the same key, then `a` followed by the inverse of `b`
    /// leaves F2L exactly as it found it.
    pub fn f2l_key(&self, last_cell: GripId) -> [u8; 72] {
        std::array::from_fn(|i| {
            if self.piece_belongs_to_cell(i, last_cell) {
                u8::MAX // don't care which last-cell piece this is
            } else {
                INDISTINGUISHABLE_ATTITUDE_CLASS[i][self.piece_attitudes[i].id() as usize]
            }
        })
    }
}

const fn indices_for_grip(g: GripId) -> [u8; 26] {
    let mut strides = [1, 3, 9, 27];
    strides.swap(g.axis(), 0);
    // Grid coordinates run from -1 to +1, and `vec4_to_index` offsets them by
    // +1, so the slice a grip points at is index 2 for positive grips and index
    // 0 for negative ones. (Getting this backwards is harmless on its own --
    // every rotation fixes `v` iff it fixes `-v`, so orientation checks can't
    // tell the difference -- but it matters as soon as anything asks *which*
    // cell a piece belongs to.)
    let init = strides[0]
        * match g.signum() {
            1 => 2,
            -1 => 0,
            _ => unreachable!(),
        };
    let mut ret = [0; 26];
    let mut i = 0;
    while i < ret.len() {
        let j = i + (i >= 13) as usize; // skip center
        let x = j % 3;
        let y = (j / 3) % 3;
        let z = (j / 9) % 3;
        ret[i] = adjust_index(init + x * strides[1] + y * strides[2] + z * strides[3]).unwrap();
        i += 1;
    }
    ret
}

fn gen_mul_elem_index_table() -> [[u8; 72]; ELEM_COUNT] {
    HYPERCUBE_ROTATIONS.map(|elem| {
        itertools::iproduct!(-1..=1, -1..=1, -1..=1, -1..=1)
            .filter_map(|(w, z, y, x)| vec4_to_index(elem * vec4(x, y, z, w)))
            .collect_array()
            .unwrap()
    })
}

fn vec4_to_index(v: Vec4) -> Option<u8> {
    adjust_index((v + vec4(1, 1, 1, 1)).dot(vec4(1, 3, 9, 27)) as usize)
}

const fn adjust_index(mut i: usize) -> Option<u8> {
    let excluded_pieces = [
        13, // O center [1, 1, 1, 0]
        31, // F center [1, 1, 0, 1]
        37, // U center [1, 0, 1, 1]
        39, // R center [0, 1, 1, 1]
        40, //   core   [1, 1, 1, 1]
        41, // L center [2, 1, 1, 1]
        43, // D center [1, 2, 1, 1]
        49, // B center [1, 1, 2, 1]
        67, // I center [1, 1, 1, 2]
    ];

    let mut j = excluded_pieces.len() - 1;
    loop {
        if i == excluded_pieces[j] {
            return None;
        }
        if i > excluded_pieces[j] {
            i -= 1;
        }

        if j == 0 {
            return Some(i as u8);
        }
        j -= 1;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::HYPERCUBE_GRIPS;

    #[test]
    fn test_all_pieces_indices() {
        for g1 in HYPERCUBE_GRIPS {
            for g2 in HYPERCUBE_GRIPS {
                if g1 == g2 {
                    continue;
                }
                let set1: HashSet<_> = HashSet::from_iter(INDICES_FOR_GRIP[g1.id() as usize]);
                let set2: HashSet<_> = HashSet::from_iter(INDICES_FOR_GRIP[g2.id() as usize]);
                let expected_intersection = if g1.axis() == g2.axis() { 0 } else { 9 };
                assert_eq!(expected_intersection, set1.intersection(&set2).count());
            }
        }
    }

    #[test]
    fn test_unoriented_pieces() {
        let mut state = PuzzleState::default();
        state.do_twists(&crate::parse_twists("R U R' U R U2 R'"));
        assert_eq!(state.unoriented_pieces(crate::D), [0, 0, 0]);
        assert_eq!(state.unoriented_pieces(crate::U), [0, 3, 6]);
    }

    #[test]
    fn test_piece_grips() {
        let counts = PIECE_GRIPS.iter().map(|g| g.len()).counts();
        assert_eq!(counts[&2], 24); // ridges (2c)
        assert_eq!(counts[&3], 32); // edges (3c)
        assert_eq!(counts[&4], 16); // corners (4c)
        assert_eq!(counts.len(), 3); // no centers or core

        // Each cell has 6 ridges, 12 edges, and 8 corners.
        for g in HYPERCUBE_GRIPS {
            let indices = INDICES_FOR_GRIP[g.id() as usize];
            for (kind, len) in [
                (CELL_RIDGES.as_slice(), 2),
                (&CELL_EDGES, 3),
                (&CELL_CORNERS, 4),
            ] {
                for &i in kind {
                    assert_eq!(PIECE_GRIPS[indices[i] as usize].len(), len);
                    assert!(PIECE_GRIPS[indices[i] as usize].contains(g));
                }
            }
        }
    }

    #[test]
    fn test_indistinguishable_attitude_classes() {
        for i in 0..72 {
            let classes = INDISTINGUISHABLE_ATTITUDE_CLASS[i].iter().counts();
            // Only ridges are ambiguous, and they have a 4-element stabilizer.
            let expected = if PIECE_GRIPS[i].len() == 2 { 4 } else { 1 };
            assert!(classes.values().all(|&n| n == expected), "piece {i}");
            assert_eq!(classes.len(), ELEM_COUNT / expected);
        }
    }

    /// Canonicalizing an attitude must not change anything anyone can see, and
    /// must survive being carried to another location by a twist -- that second
    /// property is what lets an algorithm's effect be stored as a lookup table
    /// instead of replayed.
    #[test]
    fn test_canonical_attitude() {
        for i in 0..72 {
            for id in 0..ELEM_COUNT as u8 {
                let attitude = ElemId::new(id);
                let canonical = canonical_attitude(i, attitude);

                for g in PIECE_GRIPS[i].iter() {
                    assert_eq!(attitude.inv() * g, canonical.inv() * g, "piece {i} {id}");
                }
                assert_eq!(canonical, canonical_attitude(i, canonical));

                for elem in *HYPERCUBE_ROTATIONS {
                    let moved_to = MUL_ELEM_INDEX[elem.id() as usize][i] as usize;
                    assert_eq!(
                        canonical_attitude(moved_to, elem * attitude),
                        canonical_attitude(moved_to, elem * canonical),
                        "piece {i} {id} under {elem:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn test_piece_home() {
        let mut state = PuzzleState::default();
        for i in 0..72 {
            assert_eq!(i, state.piece_home(i));
        }
        state.do_twists(&crate::parse_twists("R U IUFR RU"));
        // Every piece has a distinct home, and solved pieces are already there
        // (though a piece can be home and still be misoriented).
        assert_eq!(72, (0..72).map(|i| state.piece_home(i)).unique().count());
        assert!((0..72).any(|i| !state.is_piece_solved(i)));
        for i in 0..72 {
            assert!(!state.is_piece_solved(i) || state.piece_home(i) == i);
        }
    }

    /// Reorienting and then twisting is the same as twisting the conjugated
    /// twist and then reorienting.
    #[test]
    fn test_reorient() {
        let scramble = crate::parse_twists("R U IUFR RU IL2 F");
        let mut scrambled = PuzzleState::default();
        scrambled.do_twists(&scramble);

        for elem in *HYPERCUBE_ROTATIONS {
            assert!(PuzzleState::default().reorient(elem).is_solved());
            assert_eq!(scrambled, scrambled.reorient(elem).reorient(elem.inv()));

            let mut conjugated = PuzzleState::default();
            conjugated.do_twists(&scramble.iter().map(|&t| elem.transform(t)).collect_vec());
            assert_eq!(scrambled.reorient(elem), conjugated);
        }
    }

    /// A twist of grip `g` must move exactly the cell that `g` points at.
    #[test]
    fn test_twists_move_their_own_cell() {
        for g in HYPERCUBE_GRIPS {
            for i in INDICES_FOR_GRIP[g.id() as usize] {
                assert!(PIECE_GRIPS[i as usize].contains(g), "{g} piece {i}");
            }
        }
    }

    /// A word that is trivial on the 3^3 is also trivial on the 3^4 when
    /// written with "big 3D" moves, because every such twist fixes the W axis
    /// and acts as the same 3D move on each of the three W layers.
    #[test]
    fn test_is_solved() {
        let sexy = "R U R' U' ";
        for (should_be_solved, alg) in [
            (true, ""),
            (false, "R"),
            (false, "IU"),
            (false, "F R U R' U' F'"), // preserves F2L of a *layer*, not a cell
            (true, "R R'"),
            (true, "R L R' L'"),
            (true, &sexy.repeat(6)),
        ] {
            let mut state = PuzzleState::default();
            state.do_twists(&crate::parse_twists(alg));
            assert_eq!(should_be_solved, state.is_solved(), "{alg:?}");
        }
    }

    /// Twists of the last cell never disturb F2L, and any state agrees with
    /// solved on its F2L key exactly when F2L is intact.
    #[test]
    fn test_f2l_solved_and_key() {
        let solved_key = PuzzleState::default().f2l_key(crate::I);

        let mut state = PuzzleState::default();
        for name in ["IU", "IUR", "IU2", "IUFR"] {
            state.do_twists(&crate::parse_twists(name));
            assert!(state.is_f2l_solved(crate::I), "{name:?}");
            assert!(!state.is_solved(), "{name:?}");
            assert_eq!(solved_key, state.f2l_key(crate::I), "{name:?}");
            assert_eq!(0, state.unsolved_pieces_outside_cell(crate::I));
        }

        // A twist of any other cell does disturb F2L.
        for name in ["R", "RU", "OU", "LUF"] {
            let mut state = PuzzleState::default();
            state.do_twists(&crate::parse_twists(name));
            assert!(!state.is_f2l_solved(crate::I), "{name:?}");
            assert_ne!(solved_key, state.f2l_key(crate::I), "{name:?}");
        }
    }

    /// The whole point of the F2L key: matching keys means the two sequences
    /// differ by something that leaves F2L alone.
    #[test]
    fn test_f2l_key_finds_algs() {
        // The "4D sune": preserves F2L but permutes and reorients the last
        // cell. (`IU2` and `IB2` are the names this crate gives the half turns
        // written `ID2` and `IF2` elsewhere.)
        let alg = crate::parse_twists("RU IU2 RD IL2 RU IB2 RD");
        let (first, second) = alg.split_at(4);

        let mut a = PuzzleState::default();
        a.do_twists(first);

        // `b` walks the second half backwards from solved.
        let mut b = PuzzleState::default();
        b.do_twists(&second.iter().rev().map(|t| t.inv()).collect_vec());

        assert_eq!(a.f2l_key(crate::I), b.f2l_key(crate::I));
        assert!(!a.is_f2l_solved(crate::I)); // neither half is F2L-safe alone

        let mut whole = PuzzleState::default();
        whole.do_twists(&alg);
        assert!(whole.is_f2l_solved(crate::I));
        assert!(!whole.is_solved());
    }
}
