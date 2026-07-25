//! Solving the last cell, once blockbuilding has finished F2L.
//!
//! Blockbuilding stops when every piece outside one cell is solved. What's left
//! is a 3x3x3 cell whose 26 pieces are scrambled, and which can only be fixed
//! by sequences that leave and then restore F2L. Those sequences are
//! *algorithms*: [`algs`] finds them ahead of time by meet-in-the-middle, and
//! [`search`] strings them together.
//!
//! Everything here works in a canonical frame where the last cell is
//! [`CANONICAL_LAST_CELL`]. A real solve leaves whichever cell blockbuilding
//! chose until last, so [`PuzzleState::reorient`] rotates the puzzle into the
//! canonical frame first and [`ElemId::transform`] rotates the answer back out.

use std::fmt;

use itertools::Itertools;

use crate::sim::*;

mod algs;
mod search;

pub use algs::{Alg, AlgPass, AlgTable, AlgTableParams};
pub use search::{
    LastCellSearchParams, LastCellSolution, LastCellSolver, find_canonical_frame, solve_last_cell,
};

/// Cell that algorithms are generated for.
pub const CANONICAL_LAST_CELL: GripId = I;

/// Longest algorithm we will store.
pub const MAX_ALG_LEN: usize = 12;

/// Piece locations of the canonical last cell.
#[static_init::dynamic]
pub static CELL_PIECES: [usize; 26] =
    INDICES_FOR_GRIP[CANONICAL_LAST_CELL.id() as usize].map(|i| i as usize);

/// Grips touched by each piece of the canonical last cell.
#[static_init::dynamic]
static CELL_PIECE_GRIPS: [GripSet; 26] = CELL_PIECES.map(|i| PIECE_GRIPS[i]);

/// Maps a piece location to its index within the canonical last cell, or
/// [`u8::MAX`] if it is not in that cell.
#[static_init::dynamic]
static CELL_PIECE_INDEX: [u8; 72] = {
    let mut ret = [u8::MAX; 72];
    for (j, &i) in CELL_PIECES.iter().enumerate() {
        ret[i] = j as u8;
    }
    ret
};

/// The 1x1x3 rows of the last cell, as indices into its 26 pieces.
///
/// Each row is either `[4c, 3c, 4c]` or `[3c, 2c, 3c]`. The three rows through
/// the cell's own centre are left out, since that piece is not tracked.
///
/// These are the "vertical bars" that good last-cell algorithms move around as
/// units, which is why the search prefers states where they are already built.
#[static_init::dynamic]
pub static CELL_ROWS: [[usize; 3]; 24] = {
    // Pieces are ordered by `(z, y, x)` within the cell, skipping its centre.
    let cell_index = |x: usize, y: usize, z: usize| {
        let j = x + 3 * y + 9 * z;
        (j != 13).then(|| j - (j > 13) as usize)
    };
    itertools::iproduct!(0..3, 0..3, 0..3)
        .filter_map(|(axis, a, b)| {
            (0..3)
                .map(|k| match axis {
                    0 => cell_index(k, a, b),
                    1 => cell_index(a, k, b),
                    _ => cell_index(a, b, k),
                })
                .collect::<Option<Vec<_>>>()? // drops rows through the centre
                .try_into()
                .ok()
        })
        .collect_array()
        .expect("a 3x3x3 has 24 rows that avoid its centre")
};

/// Configuration of the 26 pieces of the last cell.
///
/// Entry `j` is the attitude of the piece currently at the cell's `j`th
/// location, so this carries both permutation and orientation. It is only
/// meaningful when F2L is intact, which is exactly when all 26 pieces of the
/// cell are somewhere in the cell.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellState(pub [ElemId; 26]);

impl Default for CellState {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl CellState {
    pub const SOLVED: Self = Self([IDENT; 26]);

    /// Reads the last cell out of a puzzle state that is already in the
    /// canonical frame.
    pub fn of(state: &PuzzleState) -> Self {
        Self(CELL_PIECES.map(|i| state.piece_attitudes()[i]))
    }

    /// Replaces each attitude with the least attitude indistinguishable from
    /// it, so that states that look identical compare equal.
    #[must_use]
    pub fn canonicalize(self) -> Self {
        Self(std::array::from_fn(|j| {
            canonical_attitude(CELL_PIECES[j], self.0[j])
        }))
    }

    pub fn is_solved(self) -> bool {
        (0..26).all(|j| self.is_piece_solved(j))
    }

    pub fn is_piece_solved(self, j: usize) -> bool {
        CELL_PIECE_GRIPS[j].iter().all(|g| self.0[j] * g == g)
    }

    /// Returns whether the piece at `j` shows its last-cell sticker outward.
    pub fn is_piece_oriented(self, j: usize) -> bool {
        self.0[j] * CANONICAL_LAST_CELL == CANONICAL_LAST_CELL
    }

    /// Returns the number of misoriented `[ridges, edges, corners]`, i.e. the
    /// 2c, 3c, and 4c pieces whose last-cell sticker does not face outward.
    ///
    /// All zeroes means OLC is done.
    pub fn unoriented(self) -> [usize; 3] {
        [
            CELL_RIDGES.as_slice(),
            CELL_EDGES.as_slice(),
            CELL_CORNERS.as_slice(),
        ]
        .map(|kind| kind.iter().filter(|&&j| !self.is_piece_oriented(j)).count())
    }

    /// Returns how many of the cell's 24 rows are built into bars.
    ///
    /// A row is a bar when its three pieces share one attitude, i.e. when they
    /// sit rigidly together and an algorithm can carry them as a unit. That is
    /// the same condition under which they would merge into a single
    /// [`crate::Block`], so this really is blockbuilding, just inside the last
    /// cell.
    ///
    /// A bar does not have to be in the right place, or even the right way up
    /// -- only internally consistent.
    pub fn bars(self) -> usize {
        CELL_ROWS.iter().filter(|row| self.is_bar(row)).count()
    }

    fn is_bar(self, row: &[usize; 3]) -> bool {
        // Every row ends in a 3c or 4c piece, and those show enough stickers to
        // pin their attitude down exactly, so one end fixes what the rest of
        // the row must match. Only the 2c piece in the middle of a `[3c, 2c,
        // 3c]` row needs comparing up to indistinguishability.
        let attitude = self.0[row[0]];
        row.iter().all(|&j| {
            canonical_attitude(CELL_PIECES[j], attitude)
                == canonical_attitude(CELL_PIECES[j], self.0[j])
        })
    }

    /// Returns the number of unsolved `[ridges, edges, corners]`.
    pub fn unsolved(self) -> [usize; 3] {
        [
            CELL_RIDGES.as_slice(),
            CELL_EDGES.as_slice(),
            CELL_CORNERS.as_slice(),
        ]
        .map(|kind| kind.iter().filter(|&&j| !self.is_piece_solved(j)).count())
    }
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [ridges, edges, corners] = self.unoriented();
        let [sr, se, sc] = self.unsolved();
        write!(
            f,
            "unoriented {ridges}/{edges}/{corners}, unsolved {sr}/{se}/{sc}",
        )
    }
}

/// What an algorithm does to the last cell, as a lookup table.
///
/// Storing the effect this way makes applying an algorithm 26 group
/// multiplications, with no need to replay its twists.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellEffect {
    /// Where the piece that ends up at each location came from.
    source: [u8; 26],
    /// The attitude each piece gains.
    attitude: [ElemId; 26],
}

impl CellEffect {
    pub const IDENTITY: Self = Self {
        source: {
            let mut ret = [0; 26];
            let mut j = 0;
            while j < 26 {
                ret[j] = j as u8;
                j += 1;
            }
            ret
        },
        attitude: [IDENT; 26],
    };

    /// Reads the effect out of the state that results from applying an
    /// algorithm to a solved puzzle.
    ///
    /// Panics if the algorithm did not restore F2L, since then the last cell's
    /// pieces are not all in the last cell and the effect is not well defined.
    pub fn of(state: &PuzzleState) -> Self {
        assert!(
            state.is_f2l_solved(CANONICAL_LAST_CELL),
            "algorithm does not preserve F2L",
        );
        Self {
            source: CELL_PIECES.map(|i| CELL_PIECE_INDEX[state.piece_home(i)]),
            attitude: CELL_PIECES.map(|i| canonical_attitude(i, state.piece_attitudes()[i])),
        }
    }

    pub fn apply(&self, state: CellState) -> CellState {
        CellState(std::array::from_fn(|j| {
            self.attitude[j] * state.0[self.source[j] as usize]
        }))
    }

    /// Returns, for `[ridges, edges, corners]`, whether this effect leaves that
    /// piece type oriented whenever it started out oriented.
    ///
    /// A piece is oriented when its attitude fixes the last cell's grip. This
    /// effect sends `state[source[j]]` to `attitude[j] * state[source[j]]`, so
    /// if the source was oriented the destination is oriented exactly when
    /// `attitude[j]` fixes the grip too -- independent of the state. Piece
    /// types never mix, so checking one type's destinations is enough.
    ///
    /// This is what lets the search stage OLC the way humans do: orient the
    /// ridges, then use only edge algorithms that keep them oriented, and so
    /// on. Note it permits *permuting* pieces of an earlier type, which is
    /// strictly more freedom than requiring them to be untouched.
    pub fn preserves_orientation(&self) -> [bool; 3] {
        [
            CELL_RIDGES.as_slice(),
            CELL_EDGES.as_slice(),
            CELL_CORNERS.as_slice(),
        ]
        .map(|kind| {
            kind.iter()
                .all(|&j| self.attitude[j] * CANONICAL_LAST_CELL == CANONICAL_LAST_CELL)
        })
    }

    /// Returns everything this effect does to the given piece types, and
    /// nothing about the others.
    ///
    /// Two algorithms with equal signatures are interchangeable for any purpose
    /// that only looks at those types, so a stage can keep just the cheapest of
    /// each and search a far smaller set.
    pub fn signature(&self, kinds: [bool; 3]) -> Vec<u16> {
        std::iter::zip(
            kinds,
            [
                CELL_RIDGES.as_slice(),
                CELL_EDGES.as_slice(),
                CELL_CORNERS.as_slice(),
            ],
        )
        .filter(|&(required, _)| required)
        .flat_map(|(_, kind)| {
            kind.iter()
                .map(|&j| (self.source[j] as u16) << 8 | self.attitude[j].id() as u16)
        })
        .collect()
    }

    /// Returns whether this effect leaves every piece of the given types
    /// completely untouched.
    pub fn fixes(&self, kinds: [bool; 3]) -> bool {
        std::iter::zip(
            kinds,
            [
                CELL_RIDGES.as_slice(),
                CELL_EDGES.as_slice(),
                CELL_CORNERS.as_slice(),
            ],
        )
        .all(|(required, kind)| {
            !required
                || kind
                    .iter()
                    .all(|&j| self.source[j] as usize == j && self.attitude[j] == IDENT)
        })
    }
}

/// Merges runs of twists that use the same grip, dropping any that cancel.
///
/// Concatenating two algorithms routinely leaves two twists of one grip next to
/// each other; the solver's metric already counts those as one move, so folding
/// them together keeps the move count honest.
pub fn simplify_twists(twists: &[Twist]) -> Vec<Twist> {
    let mut ret: Vec<Twist> = Vec::with_capacity(twists.len());
    for &twist in twists {
        match ret.last_mut() {
            Some(last) if last.grip == twist.grip => {
                // Twists apply on the left, so the later one goes first.
                last.transform = twist.transform * last.transform;
                if last.transform == IDENT {
                    ret.pop();
                }
            }
            _ => ret.push(twist),
        }
    }
    ret
}

/// Returns the inverse of a twist sequence.
pub fn invert_twists(twists: &[Twist]) -> Vec<Twist> {
    twists.iter().rev().map(|t| t.inv()).collect()
}

/// Counts a twist sequence in ETM, matching [`crate::Segment`]'s metric: twists
/// of the same grip in a row count once.
pub fn twist_count(twists: &[Twist]) -> usize {
    twists
        .iter()
        .zip(std::iter::once(None).chain(twists.iter().map(Some)))
        .filter(|(twist, previous)| previous.is_none_or(|p| p.grip != twist.grip))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_state_solved() {
        assert!(CellState::SOLVED.is_solved());
        assert_eq!([0, 0, 0], CellState::SOLVED.unoriented());
        assert_eq!([0, 0, 0], CellState::SOLVED.unsolved());

        let mut state = PuzzleState::default();
        state.do_twists(&crate::parse_twists("IU"));
        let cell = CellState::of(&state);
        assert!(!cell.is_solved());
        // A whole-cell rotation misorients nothing with respect to the last
        // cell's own axis, but it does move almost everything. The two ridges
        // on the axis `IU` turns about are left alone.
        assert_eq!([0, 0, 0], cell.unoriented());
        assert_eq!([4, 12, 8], cell.unsolved());
    }

    #[test]
    fn test_cell_effect_matches_replay() {
        let alg = crate::parse_twists("RU IU2 RD IL2 RU IB2 RD");
        let mut after_alg = PuzzleState::default();
        after_alg.do_twists(&alg);
        let effect = CellEffect::of(&after_alg);

        // Applying the effect to solved reproduces the algorithm's own result.
        assert_eq!(
            CellState::of(&after_alg).canonicalize(),
            effect.apply(CellState::SOLVED).canonicalize(),
        );

        // Applying it on top of another state matches replaying the twists.
        let setup = crate::parse_twists("IUFR IL2");
        let mut state = PuzzleState::default();
        state.do_twists(&setup);
        let before = CellState::of(&state);
        state.do_twists(&alg);
        assert_eq!(
            CellState::of(&state).canonicalize(),
            effect.apply(before).canonicalize(),
        );

        assert_eq!(
            CellState::SOLVED,
            CellEffect::IDENTITY.apply(CellState::SOLVED)
        );
        assert_eq!(before, CellEffect::IDENTITY.apply(before));
    }

    #[test]
    fn test_cell_rows() {
        // 24 rows: 12 of `[4c, 3c, 4c]` and 12 of `[3c, 2c, 3c]`.
        assert_eq!(24, CELL_ROWS.len());
        assert_eq!(24, CELL_ROWS.iter().unique().count());

        let sticker_counts = CELL_ROWS
            .iter()
            .map(|row| row.map(|j| CELL_PIECE_GRIPS[j].len()))
            .counts();
        assert_eq!(12, sticker_counts[&[4, 3, 4]]);
        assert_eq!(12, sticker_counts[&[3, 2, 3]]);

        // Every piece of the cell lies on some row.
        assert_eq!(
            26,
            CELL_ROWS.iter().flatten().unique().count(),
            "some piece is on no row",
        );
    }

    #[test]
    fn test_bars() {
        // A solved cell is all 24 bars.
        assert_eq!(24, CellState::SOLVED.bars());

        // Rotating the whole cell keeps every row rigid, so all 24 survive --
        // a bar need not be in the right place, only internally consistent.
        let mut state = PuzzleState::default();
        state.do_twists(&crate::parse_twists("IU"));
        assert_eq!(24, CellState::of(&state).bars());

        // Twisting one cell face breaks the rows that cross it.
        let mut state = PuzzleState::default();
        state.do_twists(&crate::parse_twists("RU IU2 RD IL2 RU IB2 RD"));
        let broken = CellState::of(&state);
        assert!(broken.bars() < 24, "{} bars", broken.bars());
        assert!(broken.bars() > 0, "expected some rows to survive");
    }

    #[test]
    fn test_simplify_and_count() {
        let parse = crate::parse_twists;

        assert_eq!(parse(""), simplify_twists(&parse("IU IU2 IU")));
        assert_eq!(parse("R2"), simplify_twists(&parse("R R")));
        assert_eq!(parse("IU2 R2"), simplify_twists(&parse("IU IU R R")));
        // Nothing to merge across a different grip.
        assert_eq!(parse("IU R IU"), simplify_twists(&parse("IU R IU")));

        assert_eq!(0, twist_count(&parse("")));
        assert_eq!(3, twist_count(&parse("IU R IU")));
        assert_eq!(2, twist_count(&parse("IU IU2 R"))); // same grip in a row
        assert_eq!(
            parse("R U R' U' R' F R F'"),
            invert_twists(&invert_twists(&parse("R U R' U' R' F R F'"))),
        );
    }
}
