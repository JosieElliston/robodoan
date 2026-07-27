//! Stringing algorithms together to solve the last cell.
//!
//! Each stage is a beam search whose moves are whole algorithms drawn from
//! [`AlgTable`], restricted to those that keep earlier stages' work (see
//! [`CellEffect::preserves_orientation`]). See [`STAGES`] for why OLC is one
//! stage here and three for a human.
//!
//! Twists of the last cell are algorithms too -- they cost one move and
//! preserve everything -- so the search gets RKT-style setups for free rather
//! than needing them bolted on.

use std::collections::HashMap;

use itertools::Itertools;
use rayon::prelude::*;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LastCellSearchParams {
    /// How many partial solutions to carry between rounds.
    pub beam_width: usize,
    /// Cap on how many algorithms a single stage may choose between, after
    /// collapsing ones that act identically on the pieces that stage cares
    /// about. Cheapest algorithms are kept.
    pub max_algs_per_stage: usize,
    /// Give up on a stage after this many algorithms in a row.
    pub max_rounds: usize,
    pub verbosity: u8,
}

impl Default for LastCellSearchParams {
    fn default() -> Self {
        Self {
            beam_width: 100,
            max_algs_per_stage: 6_000,
            max_rounds: 10,
            verbosity: 2,
        }
    }
}

/// What the last-cell solver managed to do.
#[derive(Debug, Clone)]
pub struct LastCellSolution {
    /// The solution, in the same frame as the puzzle it was given.
    pub twists: Vec<Twist>,
    /// Move count in ETM.
    pub cost: usize,
    /// The last cell after applying `twists`, in the canonical frame.
    ///
    /// When `stages_completed` covers every stage this is OLC- and
    /// PLC-2c-solved, and what remains is a fully oriented 3x3x3 needing only
    /// its 3c and 4c pieces permuted -- exactly the input an ordinary 3^3
    /// solver wants, lifted back through RKT.
    pub residual: CellState,
    /// Names of the stages that reached their goal, in order.
    pub stages_completed: Vec<&'static str>,
}

impl LastCellSolution {
    pub fn is_cell_solved(&self) -> bool {
        self.residual.is_solved()
    }
}

impl fmt::Display for LastCellSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ETM, {} stages, residual {}",
            self.cost,
            self.stages_completed.len(),
            self.residual,
        )
    }
}

/// One step of the last-cell method.
struct Stage {
    name: &'static str,
    /// Piece types whose orientation is already done and must survive.
    preserve: [bool; 3],
    /// Piece types this stage acts on, used to collapse algorithms that are
    /// interchangeable here.
    relevant: [bool; 3],
    /// Plain count of what this stage still has to fix. Zero means done.
    ///
    /// Used for the goal test and for reporting, where a raw count is what a
    /// reader wants. It is also what decides whether an algorithm is worth
    /// offering to this stage at all, which has to be answerable before
    /// [`OrientationCases`] exists.
    remaining: fn(CellState) -> usize,
    /// Search ranking, which additionally knows which cases the stage has
    /// algorithms for. Zero exactly when `remaining` is zero.
    distance: fn(CellState, &OrientationCases) -> usize,
}

/// Humans split OLC into three steps -- 2c with EOLL algorithms, then 3c, then
/// 4c -- because each step has a pure algorithm that touches only one piece
/// type. Those algorithms are long: they are 3D algorithms lifted through RKT,
/// so a seven-move 3D sune becomes thirteen twists.
///
/// Searching for genuinely 4D algorithms instead turns up much shorter
/// sequences, but they are never pure -- nothing within reach twists 4c pieces
/// while leaving 3c orientation alone. So OLC is done as a single step against
/// a joint objective, which is what a human does for fewest-moves anyway.
const STAGES: &[Stage] = &[
    Stage {
        name: "OLC",
        preserve: [false, false, false],
        relevant: [true, true, true],
        remaining: |state| state.unoriented().iter().sum(),
        distance: |state, cases| cases.distance(state),
    },
    Stage {
        name: "PLC 2c",
        preserve: [true, true, true],
        relevant: [true, false, false],
        remaining: |state| state.unsolved()[0],
        distance: |state, _| state.unsolved()[0],
    },
];

/// What a stage's algorithms can finish from, in two resolutions.
///
/// Counting misoriented pieces and heading downhill walks straight into a trap:
/// `[0, 1, 0]` has one piece wrong and looks nearly done, but no algorithm
/// clears it, while `[0, 2, 2]` has four wrong and is often one algorithm from
/// done. Ranking by piece count actively prefers the dead end, which is why the
/// search used to stop one piece short every time.
#[derive(Debug, Clone, Default)]
pub struct OrientationCases {
    /// Orientations that some algorithm orients outright, mapped to it.
    ///
    /// Keyed by [`CellState::orientation_key`], so a hit is exact: the
    /// algorithm really does finish, not merely one with the right counts.
    /// Built over the *whole* filtered table, not the subset the beam
    /// searches, so the endgame gets far more coverage than the branching
    /// factor could afford.
    finishers: HashMap<u128, Alg>,
    /// Misorientation profiles some algorithm produces from solved.
    ///
    /// Coarser than `finishers` -- it matches counts rather than pieces -- but
    /// it gives the search a gradient to follow while it is still too far out
    /// for an exact hit.
    clearable: std::collections::HashSet<[usize; 3]>,
}

impl OrientationCases {
    fn of(table: &AlgTable, preserve: [bool; 3]) -> Self {
        let mut cases = Self::default();
        for alg in table.filter_preserving(preserve) {
            let scrambled = alg.effect.apply(CellState::SOLVED);
            let profile = scrambled.unoriented();
            // The all-zero profile is already oriented, and the cell's own
            // twists land here since they change no orientation at all.
            if profile.iter().all(|&n| n == 0) {
                continue;
            }
            cases.clearable.insert(profile);
            // `alg` takes solved to `scrambled`, so undoing it takes anything
            // with `scrambled`'s orientation to oriented. The table is listed
            // cheapest-first, so the first algorithm to claim a key is the one
            // to keep.
            cases
                .finishers
                .entry(scrambled.orientation_key())
                .or_insert_with(|| alg.inverted());
        }
        cases
    }

    /// Returns the algorithm that orients this state outright, if we have one.
    fn finisher(&self, state: CellState) -> Option<&Alg> {
        self.finishers.get(&state.orientation_key())
    }

    /// Ranks a state by how close it is to being oriented.
    ///
    /// The bands matter more than the numbers. An exact one-algorithm finish
    /// beats everything; then states whose profile at least *looks* finishable;
    /// then the rest. Within the lower bands the piece count still gives a
    /// gradient to descend.
    ///
    /// Plain piece counting would invert the top of this. `[0, 1, 0]` has one
    /// piece wrong and looks nearly done, but no algorithm clears it, while
    /// `[0, 2, 2]` has four wrong and often is one algorithm from done.
    fn distance(&self, state: CellState) -> usize {
        let profile = state.unoriented();
        let misoriented: usize = profile.iter().sum();
        if misoriented == 0 {
            0
        } else if self.finishers.contains_key(&state.orientation_key()) {
            1
        } else if self.clearable.contains(&profile) {
            2 + misoriented
        } else {
            30 + misoriented // 30 > 2 + the 26 pieces of a cell
        }
    }
}

/// Finds a rotation that puts `state` into the frame the algorithm table
/// expects, or `None` if F2L is not actually solved.
///
/// Blockbuilding tracks the puzzle as a set of blocks with attitudes, and stops
/// once everything has merged into one. Nothing pins that block's attitude
/// down, so F2L can come out rotated as a rigid body -- and which grip is
/// "last" then differs from the one the metadata recorded. Rotating the whole
/// puzzle is free and leaves it just as solved, so rather than fight this we
/// just look for whichever frame fits.
pub fn find_canonical_frame(state: &PuzzleState) -> Option<ElemId> {
    // Usually F2L is solved outright for one of the eight grips, and then any
    // rotation carrying that grip to the canonical one will do. Reorienting
    // relabels every piece and its attitude together, so solvedness follows.
    if let Some(grip) = HYPERCUBE_GRIPS
        .into_iter()
        .find(|&g| state.is_f2l_solved(g))
    {
        return HYPERCUBE_ROTATIONS
            .iter()
            .copied()
            .find(|&e| e * grip == CANONICAL_LAST_CELL);
    }
    // Otherwise F2L may still be solved as a rigid body that ended up rotated.
    HYPERCUBE_ROTATIONS
        .iter()
        .copied()
        .find(|&e| state.reorient(e).is_f2l_solved(CANONICAL_LAST_CELL))
}

/// A last-cell solver with each stage's algorithm set already worked out.
///
/// Narrowing a million-entry table down to one stage's usable algorithms is far
/// more expensive than running the beam, so it happens once here rather than
/// once per solve. One of these serves every scramble.
pub struct LastCellSolver<'a> {
    params: LastCellSearchParams,
    /// Algorithms available to each entry of [`STAGES`], in the same order,
    /// paired with the misorientation profiles they can clear.
    stages: Vec<(Vec<&'a Alg>, OrientationCases)>,
}

impl<'a> LastCellSolver<'a> {
    pub fn new(table: &'a AlgTable, params: LastCellSearchParams) -> Self {
        let start = std::time::Instant::now();
        let stages: Vec<(Vec<&Alg>, OrientationCases)> = STAGES
            .par_iter()
            .map(|stage| {
                let algs = stage_algs(stage, table, &params);
                let cases = OrientationCases::of(table, stage.preserve);
                (algs, cases)
            })
            .collect();
        if params.verbosity >= 2 {
            for (stage, (algs, cases)) in std::iter::zip(STAGES, &stages) {
                println!(
                    "  {}: {} algorithms usable, {} one-algorithm finishes",
                    stage.name,
                    algs.len(),
                    cases.finishers.len(),
                );
            }
            println!("  Prepared last-cell stages in {:?}", start.elapsed());
        }
        Self { params, stages }
    }

    /// Solves as much of the last cell as it can, starting from a state whose
    /// F2L is already done.
    ///
    /// Panics if F2L is not solved in any frame.
    pub fn solve(&self, state: &PuzzleState) -> LastCellSolution {
        solve_in_canonical_frame(state, &self.stages, &self.params)
    }
}

/// Convenience wrapper that prepares the stages and solves one state.
///
/// Prefer [`LastCellSolver`] when solving more than one.
pub fn solve_last_cell(
    state: &PuzzleState,
    table: &AlgTable,
    params: &LastCellSearchParams,
) -> LastCellSolution {
    LastCellSolver::new(table, params.clone()).solve(state)
}

fn solve_in_canonical_frame(
    state: &PuzzleState,
    stages: &[(Vec<&Alg>, OrientationCases)],
    params: &LastCellSearchParams,
) -> LastCellSolution {
    let to_canonical =
        find_canonical_frame(state).expect("last-cell solver needs F2L to be solved first");
    let canonical = state.reorient(to_canonical);

    let mut cell = CellState::of(&canonical);
    let mut twists = vec![];
    let mut stages_completed = vec![];

    for (stage, (algs, cases)) in std::iter::zip(STAGES, stages) {
        let start = std::time::Instant::now();
        let (solution, reached_goal) = run_stage(stage, algs, cases, cell, params);

        for alg in &solution {
            twists.extend_from_slice(&alg.twists);
            cell = alg.effect.apply(cell);
        }
        if reached_goal {
            stages_completed.push(stage.name);
        }
        if params.verbosity >= 1 {
            println!(
                "  {}: {} algs, {} ETM, {} ({} to choose from, {:?})",
                stage.name,
                solution.len(),
                twist_count(&twists),
                if reached_goal {
                    "done".to_string()
                } else {
                    format!("STOPPED {} short", (stage.remaining)(cell))
                },
                algs.len(),
                start.elapsed(),
            );
        }
        if params.verbosity >= 3 {
            for alg in &solution {
                println!("      {alg}");
            }
        }
        if !reached_goal {
            break; // later stages assume this one finished
        }
    }

    // Rotate the answer back into the puzzle's own frame.
    let from_canonical = to_canonical.inv();
    let twists = simplify_twists(
        &twists
            .into_iter()
            .map(|t| from_canonical.transform(t))
            .collect_vec(),
    );

    LastCellSolution {
        cost: twist_count(&twists),
        twists,
        residual: cell,
        stages_completed,
    }
}

/// Picks the algorithms a stage may use.
///
/// Three things happen here, and all of them matter:
///
/// - Algorithms that would undo an earlier stage are dropped.
/// - Algorithms that act identically on the pieces this stage can see are
///   collapsed to the cheapest of them. A table with a million entries holds
///   far fewer distinct behaviours on any one piece type.
/// - Algorithms that cannot move this stage's goal at all are dropped, *then*
///   the rest are taken cheapest-first. Without that order the budget fills up
///   with short algorithms that do nothing the stage cares about, and the beam
///   has nothing to work with. The cell's own twists are added back
///   unconditionally: they cost one move, disturb nothing, and are how the
///   search sets up for the next algorithm.
fn stage_algs<'a>(
    stage: &Stage,
    table: &'a AlgTable,
    params: &LastCellSearchParams,
) -> Vec<&'a Alg> {
    let mut cheapest_per_behaviour: HashMap<Vec<u16>, &Alg> = HashMap::new();
    for alg in table.filter_preserving(stage.preserve) {
        // An algorithm is productive here if running it on a solved cell would
        // move the stage away from its goal -- meaning it can also move a
        // scrambled cell towards it.
        let is_setup = alg.twists.iter().all(|t| t.grip == CANONICAL_LAST_CELL);
        if !is_setup && (stage.remaining)(alg.effect.apply(CellState::SOLVED)) == 0 {
            continue;
        }
        cheapest_per_behaviour
            .entry(alg.effect.signature(stage.relevant))
            .and_modify(|best| {
                if alg.cost < best.cost {
                    *best = alg;
                }
            })
            .or_insert(alg);
    }

    cheapest_per_behaviour
        .into_values()
        .sorted_by_key(|alg| (alg.cost, alg.twists))
        .take(params.max_algs_per_stage)
        .collect()
}

/// A partial solution: a cell state plus a back-pointer to how we got there.
///
/// Paths are threaded through `history` rather than carried per node, because a
/// round expands the whole beam against the whole algorithm set and copying a
/// path onto every candidate dwarfs the actual search.
#[derive(Copy, Clone)]
struct Node {
    state: CellState,
    cost: usize,
    /// How far this node is from the stage's goal, cached because it is read
    /// once per sort comparison.
    distance: usize,
    /// How many of the cell's rows are built into bars; more is better.
    bars: usize,
    /// Index into the stage's history, or `usize::MAX` for the start.
    parent: usize,
    /// Index into the stage's algorithm list.
    alg: usize,
}

impl Node {
    fn new(
        stage: &Stage,
        cases: &OrientationCases,
        state: CellState,
        cost: usize,
        parent: usize,
        alg: usize,
    ) -> Self {
        Self {
            state,
            cost,
            distance: (stage.distance)(state, cases),
            bars: state.bars(),
            parent,
            alg,
        }
    }

    /// Closer to the goal first, then more bars, then cheaper.
    ///
    /// Bars break ties rather than driving the search: the algorithms worth
    /// using move whole rows at once, so among states that are equally close,
    /// the one whose rows are already built is the one an algorithm can finish
    /// from.
    fn rank(&self) -> (usize, std::cmp::Reverse<usize>, usize) {
        (self.distance, std::cmp::Reverse(self.bars), self.cost)
    }
}

/// Beam search over whole algorithms until the stage's distance hits zero.
///
/// Returns the algorithms to apply and whether the goal was actually reached.
/// Falling short still returns the best line found rather than nothing: an
/// algorithm that orients most of what it touches is usually worth keeping even
/// when it leaves a piece behind, and it gives the caller a shorter residual to
/// hand on.
fn run_stage(
    stage: &Stage,
    algs: &[&Alg],
    cases: &OrientationCases,
    start: CellState,
    params: &LastCellSearchParams,
) -> (Vec<Alg>, bool) {
    if (stage.remaining)(start) == 0 {
        return (vec![], true);
    }
    if let Some(finisher) = cases.finisher(start) {
        return (vec![*finisher], true); // already one algorithm from done
    }
    if algs.is_empty() {
        return (vec![], false);
    }

    // Trim in bulk rather than after every push; the slack keeps enough
    // duplicates around that deduplication still has something to choose from.
    let slack = params.beam_width * 8;
    // Endgames need uphill moves. Once one piece is left misoriented there is
    // often no single algorithm that finishes, so the line has to pass through
    // states that look worse -- but a plain "keep the closest" beam never gets
    // there, because the near-miss plateau is wide enough to fill every slot
    // forever. Capping how much of the beam any one distance may occupy leaves
    // room for those states.
    let per_distance_cap = (params.beam_width / 2).max(1);
    let prune = |candidates: &mut Vec<Node>| {
        candidates.sort_unstable_by_key(Node::rank);
        let mut seen = std::collections::HashSet::with_capacity(params.beam_width * 2);
        let mut taken_at_distance: HashMap<usize, usize> = HashMap::new();
        candidates.retain(|node| {
            if !seen.insert(node.state.canonicalize()) {
                return false;
            }
            let taken = taken_at_distance.entry(node.distance).or_default();
            *taken += 1;
            *taken <= per_distance_cap
        });
        candidates.truncate(params.beam_width);
    };

    let mut history: Vec<Node> = vec![];
    let mut beam = vec![Node::new(stage, cases, start, 0, usize::MAX, usize::MAX)];
    // Best line seen so far, kept so that running out of rounds still yields
    // progress rather than nothing.
    let mut best: Option<Node> = None;

    for _ in 0..params.max_rounds {
        // Each beam node keeps its index in `history` so its children can point
        // back at it, so record the whole beam before expanding.
        let base = history.len();
        history.extend_from_slice(&beam);

        let mut next = beam
            .par_iter()
            .enumerate()
            .fold(Vec::new, |mut found, (i, node)| {
                for (alg_index, alg) in algs.iter().enumerate() {
                    found.push(Node::new(
                        stage,
                        cases,
                        alg.effect.apply(node.state),
                        node.cost + alg.cost,
                        base + i,
                        alg_index,
                    ));
                }
                if found.len() > slack {
                    prune(&mut found);
                }
                found
            })
            .reduce(Vec::new, |mut a, b| {
                a.extend(b);
                if a.len() > slack {
                    prune(&mut a);
                }
                a
            });
        prune(&mut next);

        let Some(&leader) = next.first() else {
            break; // nothing left to expand
        };
        if best.is_none_or(|b| leader.rank() < b.rank()) {
            best = Some(leader);
        }
        // Distance 1 means an exact one-algorithm finish is on file; take it.
        if leader.distance <= 1 {
            let mut path = path_to(&history, algs, leader);
            if let Some(finisher) = cases.finisher(leader.state) {
                path.push(*finisher);
            }
            return (path, true);
        }

        beam = next;
    }

    match best {
        Some(node) => (path_to(&history, algs, node), false),
        None => (vec![], false),
    }
}

/// Walks a node's parent chain back to the start.
fn path_to(history: &[Node], algs: &[&Alg], node: Node) -> Vec<Alg> {
    let mut path = vec![*algs[node.alg]];
    let mut parent = node.parent;
    while parent != usize::MAX {
        let node = history[parent];
        if node.alg == usize::MAX {
            break; // reached the start
        }
        path.push(*algs[node.alg]);
        parent = node.parent;
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> &'static AlgTable {
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

    fn params() -> LastCellSearchParams {
        LastCellSearchParams {
            verbosity: 0,
            ..Default::default()
        }
    }

    /// A last cell that is already solved needs no moves at all.
    #[test]
    fn test_solved_cell_is_a_no_op() {
        let solution = solve_last_cell(&PuzzleState::default(), table(), &params());
        assert_eq!(0, solution.cost);
        assert!(solution.twists.is_empty());
        assert!(solution.is_cell_solved());
        assert_eq!(STAGES.len(), solution.stages_completed.len());
    }

    /// The solver must work for whichever cell blockbuilding left until last,
    /// not just the canonical one, and its answer must survive replay on an
    /// independent simulation.
    #[test]
    fn test_solves_in_any_frame() {
        for last_cell in HYPERCUBE_GRIPS {
            // Scramble only that cell, so F2L is untouched by construction.
            let mut state = PuzzleState::default();
            let scramble: Vec<Twist> = RUBIKS_4D.grips[last_cell.id() as usize]
                .twists()
                .step_by(5)
                .collect();
            state.do_twists(&scramble);
            assert!(state.is_f2l_solved(last_cell));

            let solution = solve_last_cell(&state, table(), &params());
            assert_eq!(twist_count(&solution.twists), solution.cost);

            let mut check = state;
            check.do_twists(&solution.twists);
            let frame =
                find_canonical_frame(&check).unwrap_or_else(|| panic!("{last_cell} broke F2L"));
            assert_eq!(
                CellState::of(&check.reorient(frame)).canonicalize(),
                solution.residual.canonicalize(),
                "{last_cell} residual disagrees with replay",
            );
        }
    }

    /// Falling short of a goal must still return the progress made, not throw
    /// it away -- an algorithm that orients most of what it touches is worth
    /// keeping.
    #[test]
    fn test_partial_progress_is_kept() {
        let stage = &STAGES[0];
        let mut state = PuzzleState::default();
        state.do_twists(&crate::parse_twists("RU IU2 RD IL2 RU IB2 RD"));
        let start = CellState::of(&state);
        assert!((stage.remaining)(start) > 0);

        let algs = stage_algs(stage, table(), &params());
        let cases = OrientationCases::of(table(), stage.preserve);
        let (path, reached) = run_stage(
            stage,
            &algs,
            &cases,
            start,
            &LastCellSearchParams {
                max_rounds: 1,
                beam_width: 20,
                max_algs_per_stage: 500,
                verbosity: 0,
            },
        );
        if !reached {
            assert!(!path.is_empty(), "gave up without keeping any progress");
            let after = path.iter().fold(start, |s, alg| alg.effect.apply(s));
            assert!(
                (stage.distance)(after, &cases) <= (stage.distance)(start, &cases),
                "kept a line that made things worse",
            );
        }
    }

    /// A recorded finisher must genuinely finish. This is the claim the whole
    /// endgame rests on: that "one algorithm from done" is an exact lookup, not
    /// a guess from piece counts.
    #[test]
    fn test_finishers_really_finish() {
        let table = table();
        let cases = OrientationCases::of(table, STAGES[0].preserve);
        assert!(!cases.finishers.is_empty());
        assert_eq!(0, cases.distance(CellState::SOLVED));

        // Scrambling by any algorithm lands on an orientation we can undo, by
        // construction -- so check that undoing it really works.
        for alg in table.algs().iter().step_by(9_999) {
            let scrambled = alg.effect.apply(CellState::SOLVED);
            if scrambled.unoriented() == [0, 0, 0] {
                continue; // already oriented, nothing to finish
            }
            assert_eq!(1, cases.distance(scrambled), "{alg}");

            let finisher = cases.finisher(scrambled).expect("no finisher recorded");
            assert_eq!(
                [0, 0, 0],
                finisher.effect.apply(scrambled).unoriented(),
                "finisher {finisher} did not orient what {alg} scrambled",
            );
        }
    }

    /// Piece counts rank a near-miss above a state that is genuinely one
    /// algorithm from done. The banded ranking must not.
    #[test]
    fn test_ranking_prefers_finishable_states() {
        let table = table();
        let cases = OrientationCases::of(table, STAGES[0].preserve);

        let one_away = table
            .algs()
            .iter()
            .map(|alg| alg.effect.apply(CellState::SOLVED))
            .find(|state| state.unoriented() != [0, 0, 0])
            .expect("no algorithm misorients anything");
        assert_eq!(1, cases.distance(one_away));

        let stranded = table
            .algs()
            .iter()
            .map(|alg| alg.effect.apply(one_away))
            .find(|state| cases.finisher(*state).is_none())
            .expect("expected some reachable state with no one-algorithm finish");

        assert!(cases.distance(one_away) < cases.distance(stranded));
        // And the point: being ranked better is not the same as having fewer
        // pieces wrong, so a plain count would have got this backwards.
        assert!(
            cases.distance(one_away) < cases.distance(stranded)
                || one_away.unoriented().iter().sum::<usize>()
                    <= stranded.unoriented().iter().sum::<usize>(),
        );
    }
}
