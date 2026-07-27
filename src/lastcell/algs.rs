//! Finding algorithms that leave F2L exactly as they found it.
//!
//! An algorithm here is any twist sequence that restores every piece outside
//! the last cell while doing something to the pieces inside it. Searching for
//! those directly is hopeless -- with 23 twists per grip the tree is far too
//! wide -- so instead we meet in the middle.
//!
//! The trick is [`PuzzleState::f2l_key`], which describes where the F2L pieces
//! are while saying nothing about the last cell. If two sequences reach states
//! with the same key, they have moved F2L to exactly the same place, so the
//! first followed by the reverse of the second puts F2L back and touches only
//! the last cell. Enumerating to depth `d` from one side therefore yields
//! algorithms up to `2d` twists long.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{BuildHasher, RandomState};

use itertools::Itertools;
use rayon::prelude::*;

use super::*;
use crate::StackVec;

/// Half of [`MAX_ALG_LEN`]: the deepest each side of the meet-in-the-middle can
/// go.
const MAX_HALF_DEPTH: usize = MAX_ALG_LEN / 2;

/// One F2L-preserving algorithm.
///
/// Kept allocation-free: a table runs to millions of these, and a `Vec` per
/// algorithm costs more in pointer-chasing and allocator traffic than the
/// twists themselves.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Alg {
    pub twists: StackVec<Twist, MAX_ALG_LEN>,
    pub effect: CellEffect,
    /// Move count in ETM.
    pub cost: usize,
}

impl Alg {
    /// Returns the algorithm that undoes this one.
    ///
    /// Costs the same, and is usually in the table already -- but not always,
    /// so the effect is recomputed rather than looked up.
    pub fn inverted(&self) -> Self {
        build_alg(invert_twists(&self.twists)).expect("undoing an algorithm preserves F2L")
    }
}

impl fmt::Display for Alg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:2} ETM  {}", self.cost, self.twists.iter().join(" "))
    }
}

/// One sweep of the meet-in-the-middle search.
///
/// A pass using a single side grip goes deep cheaply, but it can never
/// misorient a 2c piece -- the 4D echo of 3D edge orientation surviving
/// `<R, L, U, D>`. A pass with two side grips can, but has to stay shallower.
/// Running both and merging gets the strengths of each.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlgPass {
    /// Grips that algorithms may use besides the last cell's own grip.
    ///
    /// Twists of the last cell are always allowed; they are what make
    /// algorithms do anything interesting, and they cost nothing in F2L damage.
    /// The table is closed under the 24 rotations that fix the last cell
    /// afterwards, so one side grip here already yields algorithms on all six.
    pub grips: Vec<GripId>,
    /// How deep to search from each side. Algorithms are up to twice this long.
    pub half_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlgTableParams {
    pub passes: Vec<AlgPass>,
    /// Cap on how many sequences from one bucket get paired up, so that a
    /// single huge bucket cannot dominate the run.
    pub max_bucket: usize,
    pub verbosity: u8,
}

impl Default for AlgTableParams {
    fn default() -> Self {
        Self {
            passes: vec![
                AlgPass {
                    grips: vec![R],
                    half_depth: 4,
                },
                AlgPass {
                    grips: vec![R, U],
                    half_depth: 3,
                },
            ],
            max_bucket: 512,
            verbosity: 2,
        }
    }
}

/// A collection of algorithms, at most one per distinct effect on the last
/// cell.
#[derive(Debug, Clone, Default)]
pub struct AlgTable {
    algs: Vec<Alg>,
}

impl AlgTable {
    pub fn algs(&self) -> &[Alg] {
        &self.algs
    }

    pub fn len(&self) -> usize {
        self.algs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.algs.is_empty()
    }

    /// Returns the algorithms that keep each required piece type oriented,
    /// cheapest first.
    ///
    /// This is how the staged search keeps earlier work: once the ridges are
    /// oriented, only algorithms that keep them oriented may be used.
    ///
    /// The table is stored in cost order, so this preserves it without sorting.
    pub fn filter_preserving(&self, required: [bool; 3]) -> impl Iterator<Item = &Alg> {
        self.algs.iter().filter(move |alg| {
            std::iter::zip(alg.effect.preserves_orientation(), required)
                .all(|(preserved, need)| preserved || !need)
        })
    }

    /// Finds every algorithm reachable by meeting in the middle, then closes
    /// the result under the rotations that fix the last cell.
    pub fn generate(params: &AlgTableParams) -> Self {
        let start = std::time::Instant::now();
        let mut all: HashMap<CellEffect, Alg> = HashMap::new();

        for pass in &params.passes {
            assert!(
                pass.half_depth <= MAX_HALF_DEPTH,
                "half_depth must be at most {MAX_HALF_DEPTH}",
            );
            assert!(
                !pass.grips.contains(&CANONICAL_LAST_CELL),
                "the last cell's grip is always included",
            );

            let t = std::time::Instant::now();
            let buckets = enumerate_halves(pass);
            let sequences: usize = buckets.values().map(Vec::len).sum();
            let classes = buckets.len();

            let joined = join_halves(params, buckets);
            let found = joined.len();

            for (effect, alg) in close_under_rotations(joined) {
                insert_cheapest(&mut all, effect, alg);
            }

            if params.verbosity >= 2 {
                println!(
                    "  Pass {:?} depth {}: {sequences} sequences, {classes} F2L classes, \
                     {found} algorithms, {} after rotations ({:?})",
                    pass.grips.iter().map(|g| g.char()).collect::<String>(),
                    pass.half_depth,
                    all.len(),
                    t.elapsed(),
                );
            }
        }

        let algs = all
            .into_values()
            .sorted_by_key(|alg| alg.cost)
            .collect_vec();
        if params.verbosity >= 1 {
            println!(
                "  Alg table: {} algorithms in {:?}",
                algs.len(),
                start.elapsed(),
            );
        }

        Self { algs }
    }
}

/// Groups every sequence up to `half_depth` twists by where it leaves F2L.
///
/// Buckets are keyed by a hash rather than the full 72-byte key; a collision
/// only costs us a candidate that fails verification later.
fn enumerate_halves(pass: &AlgPass) -> HashMap<u64, Vec<StackVec<Twist, MAX_HALF_DEPTH>>> {
    let puzzle = &*RUBIKS_4D;
    let grips: Vec<GripId> = std::iter::once(CANONICAL_LAST_CELL)
        .chain(pass.grips.iter().copied())
        .collect();
    let twists_for_grip: Vec<Vec<Twist>> = grips
        .iter()
        .map(|&g| puzzle.grips[g.id() as usize].twists().collect())
        .collect();
    let hasher = RandomState::new();

    // Each starting twist is an independent subtree, so fan those out.
    let roots: Vec<Twist> = twists_for_grip.concat();
    let subtrees: Vec<HashMap<u64, Vec<StackVec<Twist, MAX_HALF_DEPTH>>>> = roots
        .par_iter()
        .map(|&root| {
            let mut buckets = HashMap::new();
            let mut state = PuzzleState::default();
            state.do_twist(root);
            let sequence = StackVec::from_iter([root]).unwrap();
            enumerate_from(
                pass,
                &twists_for_grip,
                &hasher,
                &mut buckets,
                state,
                sequence,
            );
            buckets
        })
        .collect();

    let mut buckets = HashMap::new();
    // The empty sequence is a valid half, and pairing it with another half is
    // what turns that half into an algorithm on its own.
    buckets.insert(
        hasher.hash_one(PuzzleState::default().f2l_key(CANONICAL_LAST_CELL)),
        vec![StackVec::new()],
    );
    for subtree in subtrees {
        for (key, sequences) in subtree {
            buckets.entry(key).or_default().extend(sequences);
        }
    }
    buckets
}

fn enumerate_from(
    pass: &AlgPass,
    twists_for_grip: &[Vec<Twist>],
    hasher: &RandomState,
    buckets: &mut HashMap<u64, Vec<StackVec<Twist, MAX_HALF_DEPTH>>>,
    state: PuzzleState,
    sequence: StackVec<Twist, MAX_HALF_DEPTH>,
) {
    let key = hasher.hash_one(state.f2l_key(CANONICAL_LAST_CELL));
    buckets.entry(key).or_default().push(sequence);

    if sequence.len() >= pass.half_depth {
        return;
    }
    let last_grip = sequence.last().map(|t| t.grip);
    for twists in twists_for_grip {
        // Two twists of one grip in a row are just a single twist we will also
        // visit, so skip them.
        if Some(twists[0].grip) == last_grip {
            continue;
        }
        for &twist in twists {
            let mut next = state;
            next.do_twist(twist);
            enumerate_from(
                pass,
                twists_for_grip,
                hasher,
                buckets,
                next,
                sequence.push(twist).expect("sequence too long"),
            );
        }
    }
}

/// Pairs up the sequences within each bucket, keeping the cheapest algorithm
/// found for each distinct effect on the last cell.
fn join_halves(
    params: &AlgTableParams,
    buckets: HashMap<u64, Vec<StackVec<Twist, MAX_HALF_DEPTH>>>,
) -> HashMap<CellEffect, Alg> {
    let per_bucket: Vec<HashMap<CellEffect, Alg>> = buckets
        .into_par_iter()
        .filter(|(_, sequences)| sequences.len() >= 2)
        .map(|(_, mut sequences)| {
            // Prefer short halves; they make short algorithms.
            sequences.sort_unstable_by_key(|s| s.len());
            sequences.truncate(params.max_bucket);

            let mut found = HashMap::new();
            for (first, second) in sequences.iter().tuple_combinations() {
                // Both orders are worth having: they are inverses of each
                // other, and have different effects.
                for [a, b] in [[first, second], [second, first]] {
                    if let Some(alg) = build_alg(a.iter().copied().chain(invert_twists(b))) {
                        insert_cheapest(&mut found, alg.effect, alg);
                    }
                }
            }
            found
        })
        .collect();

    let mut algs = HashMap::new();
    for bucket in per_bucket {
        for (effect, alg) in bucket {
            insert_cheapest(&mut algs, effect, alg);
        }
    }
    algs
}

/// Verifies a candidate and works out what it does to the last cell.
///
/// Returns `None` if it does not actually preserve F2L (the bucket key is a
/// hash, so collisions reach here) or if it does nothing at all.
fn build_alg(twists: impl IntoIterator<Item = Twist>) -> Option<Alg> {
    let twists = simplify_twists(&twists.into_iter().collect_vec());
    if twists.is_empty() {
        return None;
    }
    let twists = StackVec::from_slice(&twists)?; // too long to store
    let mut state = PuzzleState::default();
    state.do_twists(&twists);
    if !state.is_f2l_solved(CANONICAL_LAST_CELL) {
        return None;
    }
    let effect = CellEffect::of(&state);
    if effect == CellEffect::IDENTITY {
        return None;
    }
    Some(Alg {
        cost: twist_count(&twists),
        twists,
        effect,
    })
}

fn insert_cheapest(algs: &mut HashMap<CellEffect, Alg>, effect: CellEffect, alg: Alg) {
    match algs.entry(effect) {
        Entry::Occupied(mut e) => {
            if alg.cost < e.get().cost {
                e.insert(alg);
            }
        }
        Entry::Vacant(e) => {
            e.insert(alg);
        }
    }
}

/// Rotating the whole puzzle about the last cell costs nothing and turns each
/// algorithm into 23 more, using the other grips.
fn close_under_rotations(algs: HashMap<CellEffect, Alg>) -> HashMap<CellEffect, Alg> {
    let rotations: Vec<ElemId> = HYPERCUBE_ROTATIONS
        .iter()
        .copied()
        .filter(|&e| e * CANONICAL_LAST_CELL == CANONICAL_LAST_CELL)
        .collect();

    let rotated: Vec<HashMap<CellEffect, Alg>> = algs
        .into_par_iter()
        .map(|(_, alg)| {
            let mut found = HashMap::new();
            for &elem in &rotations {
                if let Some(rotated) = build_alg(alg.twists.iter().map(|&t| elem.transform(t))) {
                    insert_cheapest(&mut found, rotated.effect, rotated);
                }
            }
            found
        })
        .collect();

    let mut all = HashMap::new();
    for bucket in rotated {
        for (effect, alg) in bucket {
            insert_cheapest(&mut all, effect, alg);
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two side grips, because no algorithm on one grip alone can misorient a
    /// ridge -- the analogue of 3D edge orientation surviving `<R, L, U, D>`.
    fn small_table() -> &'static AlgTable {
        static TABLE: std::sync::OnceLock<AlgTable> = std::sync::OnceLock::new();
        TABLE.get_or_init(|| {
            AlgTable::generate(&AlgTableParams {
                passes: vec![AlgPass {
                    grips: vec![R, U],
                    half_depth: 3,
                }],
                verbosity: 0,
                ..Default::default()
            })
        })
    }

    /// Every algorithm in the table must actually restore F2L and actually do
    /// something, and its recorded effect must match replaying its twists.
    #[test]
    fn test_generated_algs_are_valid() {
        let table = small_table();
        assert!(!table.is_empty());

        for alg in table.algs() {
            let mut state = PuzzleState::default();
            state.do_twists(&alg.twists);
            assert!(state.is_f2l_solved(CANONICAL_LAST_CELL), "{alg}");
            assert!(!state.is_solved(), "{alg}");
            assert_eq!(twist_count(&alg.twists), alg.cost, "{alg}");
            assert_eq!(
                CellState::of(&state).canonicalize(),
                alg.effect.apply(CellState::SOLVED).canonicalize(),
                "{alg}",
            );
            // Simplified, so no grip appears twice in a row.
            assert!(
                alg.twists.windows(2).all(|w| w[0].grip != w[1].grip),
                "{alg}"
            );
        }
    }

    /// Closing under rotations must reach all six non-last-cell grips, not just
    /// the one the search used.
    #[test]
    fn test_table_covers_all_grips() {
        let table = small_table();
        let grips: std::collections::HashSet<GripId> = table
            .algs()
            .iter()
            .flat_map(|alg| alg.twists.iter().map(|t| t.grip))
            .collect();
        for g in HYPERCUBE_GRIPS {
            // Everything but the last cell's opposite, which no twist here can
            // reach: `O` twists move the last cell's own pieces nowhere useful
            // without a second grip to work against.
            if g != O {
                assert!(grips.contains(&g), "no algorithm uses {g}");
            }
        }
    }

    /// An algorithm that "preserves" a piece type must keep that type oriented
    /// starting from *any* state where it was already oriented, not just from
    /// solved.
    #[test]
    fn test_filter_preserving() {
        let table = small_table();

        // A state whose ridges are all oriented but which is otherwise messy.
        let mut messy = PuzzleState::default();
        messy.do_twists(&crate::parse_twists("IU IUFR IL2 IUR"));
        let messy = CellState::of(&messy);
        assert_eq!(0, messy.unoriented()[0]);
        assert_ne!([0, 0, 0], messy.unsolved());

        let preserving = table.filter_preserving([true, false, false]).collect_vec();
        assert!(!preserving.is_empty());
        for alg in &preserving {
            assert_eq!(0, alg.effect.apply(messy).unoriented()[0], "{alg}");
            assert_eq!(
                0,
                alg.effect.apply(CellState::SOLVED).unoriented()[0],
                "{alg}"
            );
        }

        // And the ones we filtered out must genuinely be unsafe here.
        let unsafe_count = table.algs().len() - preserving.len();
        assert!(unsafe_count > 0);

        // Cost order is preserved, which the stage setup relies on.
        assert!(preserving.windows(2).all(|w| w[0].cost <= w[1].cost));
    }
}
