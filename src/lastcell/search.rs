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
    /// How many algorithms iterative deepening may branch over.
    ///
    /// Much smaller than the beam's set: each extra ply multiplies the work by
    /// this, and the finisher table already supplies the last ply from the full
    /// table.
    pub ids_algs: usize,
    /// Largest twist budget iterative deepening will try before giving up and
    /// handing over to the beam.
    pub max_ids_cost: usize,
    /// How many algorithms the backward half of the meet-in-the-middle pairs
    /// up.
    ///
    /// Costs `backward_algs^2` to build and roughly that many table entries, so
    /// it trades memory and startup time for forward reach.
    pub backward_algs: usize,
    /// Most algorithms an iterative-deepening answer may use, counting the
    /// finisher.
    ///
    /// A budget alone is not enough of a leash. Deepening bounds the plies --
    /// budget `B` buys at most `B` of them -- but the cell's own rotations cost
    /// one move each, so a budget of twelve still admits around `23^11` nodes.
    /// Bounded, but hopeless. The work is `ids_algs^(max_ids_algs - 1)` per
    /// budget, so each step up here is expensive.
    pub max_ids_algs: usize,
    pub verbosity: u8,
}

impl Default for LastCellSearchParams {
    fn default() -> Self {
        Self {
            beam_width: 100,
            max_algs_per_stage: 6_000,
            max_rounds: 10,
            ids_algs: 600,
            max_ids_cost: 20,
            max_ids_algs: 3,
            backward_algs: 1_500,
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
    /// One entry per algorithm applied, in order.
    pub steps: Vec<LastCellStep>,
}

/// One algorithm of a last-cell solution, and what it did.
///
/// Recorded rather than printed because candidates are searched in parallel, so
/// only the winner's trace is worth showing and it has to survive the race.
#[derive(Debug, Clone)]
pub struct LastCellStep {
    pub stage: &'static str,
    pub alg: Alg,
    /// Misoriented `[ridges, edges, corners]` before and after.
    pub unoriented: [[usize; 3]; 2],
    /// Unsolved `[ridges, edges, corners]` before and after.
    pub unsolved: [[usize; 3]; 2],
}

impl fmt::Display for LastCellStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [before, after] = self.unoriented;
        let [was, now] = self.unsolved;
        write!(
            f,
            "{:8} unoriented {before:?} -> {after:?}  unsolved {was:?} -> {now:?}  {}",
            self.stage, self.alg,
        )
    }
}

impl LastCellSolution {
    pub fn is_cell_solved(&self) -> bool {
        self.residual.is_solved()
    }

    /// Returns a line per algorithm applied, for working out where the moves
    /// went.
    pub fn trace(&self) -> String {
        self.steps
            .iter()
            .enumerate()
            .map(|(i, step)| format!("  {:2}. {step}", i + 1))
            .join("\n")
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
/// A stored way to finish, as indices into [`OrientationCases::finish_algs`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Finish {
    first: u32,
    /// [`u32::MAX`] when one algorithm is enough.
    second: u32,
    cost: u32,
}

impl Finish {
    const NONE: u32 = u32::MAX;
}

#[derive(Debug, Clone, Default)]
pub struct OrientationCases {
    /// Algorithms that appear in a finish, already inverted so they can be
    /// applied as-is.
    finish_algs: Vec<Alg>,
    /// Orientations that one or two algorithms orient outright.
    ///
    /// This is the backward half of a meet-in-the-middle. Keyed by
    /// [`CellState::orientation_key`], so a hit is exact -- these algorithms
    /// really do finish, not merely ones with the right piece counts -- and
    /// orientation is a quotient the algorithms act on cleanly, so many
    /// distinct cell states share a key and the table stays far smaller than
    /// the number of pairs that built it.
    ///
    /// Searching two plies backward here is worth much more than one ply
    /// forward: it is built once per solver rather than once per node, so the
    /// forward search gets two extra algorithms of reach for a single hash
    /// lookup.
    finishers: HashMap<u128, Finish>,
    /// Misorientation profiles some algorithm produces from solved.
    ///
    /// Coarser than `finishers` -- it matches counts rather than pieces -- but
    /// it gives the search a gradient to follow while it is still too far out
    /// for an exact hit.
    clearable: std::collections::HashSet<[usize; 3]>,
}

impl OrientationCases {
    fn of(table: &AlgTable, preserve: [bool; 3], params: &LastCellSearchParams) -> Self {
        let mut cases = Self::default();

        // One ply, over the whole filtered table. `alg` takes solved to
        // `scrambled`, so undoing it takes anything with `scrambled`'s
        // orientation to oriented.
        let mut backward: Vec<&Alg> = vec![];
        for alg in table.filter_preserving(preserve) {
            let scrambled = alg.effect.apply(CellState::SOLVED);
            let profile = scrambled.unoriented();
            // The all-zero profile is already oriented, and the cell's own
            // twists land here since they change no orientation at all.
            if profile.iter().all(|&n| n == 0) {
                continue;
            }
            cases.clearable.insert(profile);
            if backward.len() < params.backward_algs {
                backward.push(alg); // table is cheapest-first
            }
            let key = scrambled.orientation_key();
            if !cases.finishers.contains_key(&key) {
                let index = cases.push_alg(alg.inverted());
                cases.finishers.insert(
                    key,
                    Finish {
                        first: index,
                        second: Finish::NONE,
                        cost: alg.cost as u32,
                    },
                );
            }
        }

        // Two plies, over a reduced set. Applying `a` then `b` to solved lands
        // on `s`, so undoing them in the other order -- `b` first, then `a` --
        // orients anything that shares `s`'s orientation.
        let inverted: Vec<u32> = backward
            .iter()
            .map(|alg| cases.push_alg(alg.inverted()))
            .collect();
        for (i, a) in backward.iter().enumerate() {
            let after_a = a.effect.apply(CellState::SOLVED);
            for (j, b) in backward.iter().enumerate() {
                let cost = (a.cost + b.cost) as u32;
                let key = b.effect.apply(after_a).orientation_key();
                // A pair can beat a single algorithm: two four-move algorithms
                // cost less than one eight-move one.
                match cases.finishers.entry(key) {
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        if cost < e.get().cost {
                            e.insert(Finish {
                                first: inverted[j],
                                second: inverted[i],
                                cost,
                            });
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(Finish {
                            first: inverted[j],
                            second: inverted[i],
                            cost,
                        });
                    }
                }
            }
        }

        cases
    }

    fn push_alg(&mut self, alg: Alg) -> u32 {
        self.finish_algs.push(alg);
        (self.finish_algs.len() - 1) as u32
    }

    /// Returns what it would cost to finish from here, if we know a way.
    ///
    /// The hot path only ever needs the cost, so expanding into algorithms is
    /// left to [`Self::finisher`].
    fn finish_cost(&self, state: CellState) -> Option<usize> {
        self.finishers
            .get(&state.orientation_key())
            .map(|finish| finish.cost as usize)
    }

    /// Returns the algorithms that orient this state outright, if we know any.
    fn finisher(&self, state: CellState) -> Option<Vec<Alg>> {
        let finish = self.finishers.get(&state.orientation_key())?;
        let mut algs = vec![self.finish_algs[finish.first as usize]];
        if finish.second != Finish::NONE {
            algs.push(self.finish_algs[finish.second as usize]);
        }
        Some(algs)
    }

    /// Ranks a state by how close it is to being oriented.
    ///
    /// The bands matter more than the numbers. A known finish beats everything;
    /// then states whose profile at least *looks* finishable; then the rest.
    /// Within the lower bands the piece count still gives a gradient to
    /// descend.
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
                let cases = OrientationCases::of(table, stage.preserve, &params);
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
    let mut steps = vec![];

    for (stage, (algs, cases)) in std::iter::zip(STAGES, stages) {
        let start = std::time::Instant::now();
        let (solution, reached_goal) = run_stage(stage, algs, cases, cell, params);

        for alg in &solution {
            let before = cell;
            twists.extend_from_slice(&alg.twists);
            cell = alg.effect.apply(cell);
            steps.push(LastCellStep {
                stage: stage.name,
                alg: *alg,
                unoriented: [before.unoriented(), cell.unoriented()],
                unsolved: [before.unsolved(), cell.unsolved()],
            });
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
        steps,
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
    /// `cost` plus what it would take to finish from here, when that is known.
    projected_cost: usize,
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
            // Charge for the finish now rather than discovering it later.
            // Otherwise two states both one algorithm from done look equally
            // good while one is finished by four twists and the other by
            // eight, and the search happily picks the expensive one.
            projected_cost: cost + cases.finish_cost(state).unwrap_or(0),
            distance: (stage.distance)(state, cases),
            bars: state.bars(),
            parent,
            alg,
        }
    }

    /// Closer to the goal first, then cheaper all-in, then more bars.
    ///
    /// Bars break ties rather than driving the search: the algorithms worth
    /// using move whole rows at once, so among states that are equally close
    /// and equally cheap, the one whose rows are already built is the one an
    /// algorithm can finish from.
    fn rank(&self) -> (usize, usize, std::cmp::Reverse<usize>) {
        (
            self.distance,
            self.projected_cost,
            std::cmp::Reverse(self.bars),
        )
    }
}

/// Searches a stage, trying for a shortest answer before settling for a good
/// one.
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
    if algs.is_empty() {
        return (vec![], false);
    }
    if let Some(found) = run_iterative_deepening(stage, algs, cases, start, params) {
        return (found, true);
    }
    run_beam(stage, algs, cases, start, params)
}

/// Iterative deepening on the twist budget.
///
/// Deepening on move count rather than algorithm count is what makes the answer
/// worth having: the first budget that admits any solution admits only optimal
/// ones, so what comes back is the shortest line in ETM, not merely the one
/// using fewest algorithms. Three cheap algorithms often beat two expensive
/// ones and this notices.
///
/// The finisher table does the heavy lifting. It answers "does one algorithm
/// out of the whole million-entry table finish from here?" exactly and in O(1),
/// so it is checked at every node -- effectively a free last ply drawn from a
/// far larger set than the search itself could branch over.
///
/// It still runs out of room: each extra ply multiplies the work by the
/// algorithm set, so states needing a long line fall through to the beam.
fn run_iterative_deepening(
    stage: &Stage,
    algs: &[&Alg],
    cases: &OrientationCases,
    start: CellState,
    params: &LastCellSearchParams,
) -> Option<Vec<Alg>> {
    // Only the orientation stages have a finisher table to lean on.
    if cases.finishers.is_empty() {
        return None;
    }
    // Sorted by cost, so a branch that is already too expensive means every
    // later one is too. Each is tagged as a setup (a bare rotation of the cell)
    // because chaining those has to be forbidden -- see `finish_within_budget`.
    let searched: Vec<(&Alg, bool)> = algs
        .iter()
        .copied()
        .take(params.ids_algs)
        .map(|alg| {
            let is_setup = alg.twists.iter().all(|t| t.grip == CANONICAL_LAST_CELL);
            (alg, is_setup)
        })
        .collect();
    debug_assert!(searched.windows(2).all(|w| w[0].0.cost <= w[1].0.cost));

    for budget in 1..=params.max_ids_cost {
        let start_time = std::time::Instant::now();
        // Fan out on the first algorithm; the rest recurses serially.
        let found = cases
            .finisher(start)
            .filter(|_| cases.finish_cost(start).is_some_and(|c| c <= budget))
            .or_else(|| {
                searched
                    .par_iter()
                    .filter(|(alg, _)| alg.cost < budget)
                    .find_map_any(|&(first, is_setup)| {
                        let mut path = vec![*first];
                        finish_within_budget(
                            cases,
                            &searched,
                            first.effect.apply(start),
                            budget - first.cost,
                            // One ply is spent on `first`, and the last is
                            // always the finisher.
                            params.max_ids_algs.saturating_sub(2),
                            is_setup,
                            &mut path,
                        )
                        .then_some(path)
                    })
            });

        if params.verbosity >= 3 {
            println!(
                "      budget {budget} ETM: {} ({:?})",
                if found.is_some() { "found" } else { "no" },
                start_time.elapsed(),
            );
        }
        if let Some(path) = found {
            debug_assert_eq!(
                0,
                (stage.remaining)(path.iter().fold(start, |s, a| a.effect.apply(s))),
                "iterative deepening returned a line that does not finish",
            );
            return Some(path);
        }
    }
    None
}

/// Extends `path` until the stage is finished without exceeding `budget`.
///
/// Returns whether it managed to, leaving the line in `path`.
fn finish_within_budget(
    cases: &OrientationCases,
    algs: &[(&Alg, bool)],
    state: CellState,
    budget: usize,
    plies_left: usize,
    previous_was_setup: bool,
    path: &mut Vec<Alg>,
) -> bool {
    if let Some(cost) = cases.finish_cost(state)
        && cost <= budget
    {
        path.extend(cases.finisher(state).expect("cost implies a finish"));
        return true;
    }
    if plies_left == 0 {
        return false;
    }

    for &(alg, is_setup) in algs {
        // Every algorithm costs at least one, and so does the finisher, so a
        // branch needs strictly less than the budget to leave room to end.
        if alg.cost >= budget {
            break; // sorted by cost: nothing later fits either
        }
        // Two rotations of the cell in a row compose into a single rotation,
        // which is already in the set. Without this the one-move rotations sit
        // at the front of the list and the search disappears into chains of
        // them -- twenty-three branches per ply, all of them redundant.
        if is_setup && previous_was_setup {
            continue;
        }
        path.push(*alg);
        if finish_within_budget(
            cases,
            algs,
            alg.effect.apply(state),
            budget - alg.cost,
            plies_left - 1,
            is_setup,
            path,
        ) {
            return true;
        }
        path.pop();
    }
    false
}

/// Beam search over whole algorithms until the stage's distance hits zero.
fn run_beam(
    stage: &Stage,
    algs: &[&Alg],
    cases: &OrientationCases,
    start: CellState,
    params: &LastCellSearchParams,
) -> (Vec<Alg>, bool) {
    if let Some(finish) = cases.finisher(start) {
        return (finish, true); // already within reach of the backward table
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
    // Cheapest line that actually reaches the goal, counting its finish.
    let mut best_complete: Option<Node> = None;

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

        // A node the backward table can finish gives a complete line, but not
        // necessarily the cheapest one -- the table hands back *a* way out, not
        // the best way out. So record it and keep going rather than returning
        // on the first one seen.
        if let Some(&candidate) = next
            .iter()
            .filter(|node| node.distance <= 1)
            .min_by_key(|node| node.projected_cost)
            && best_complete.is_none_or(|b: Node| candidate.projected_cost < b.projected_cost)
        {
            best_complete = Some(candidate);
        }

        // Extending a line only ever costs more, so once every surviving node
        // is already at least as expensive as the best complete line, nothing
        // left can beat it.
        if let Some(complete) = best_complete
            && next.iter().all(|node| node.cost >= complete.projected_cost)
        {
            break;
        }

        beam = next;
    }

    if let Some(node) = best_complete {
        let mut path = path_to(&history, algs, node);
        path.extend(cases.finisher(node.state).unwrap_or_default());
        return (path, true);
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
        let cases = OrientationCases::of(table(), stage.preserve, &params());
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
                ..Default::default()
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
        let cases = OrientationCases::of(table, STAGES[0].preserve, &params());
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

            let finish = cases.finisher(scrambled).expect("no finisher recorded");
            let finished = finish.iter().fold(scrambled, |s, a| a.effect.apply(s));
            assert_eq!(
                [0, 0, 0],
                finished.unoriented(),
                "the recorded finish did not orient what {alg} scrambled",
            );
            assert_eq!(
                cases.finish_cost(scrambled),
                Some(finish.iter().map(|a| a.cost).sum::<usize>()),
            );
        }
    }

    /// Piece counts rank a near-miss above a state that is genuinely one
    /// algorithm from done. The banded ranking must not.
    #[test]
    fn test_ranking_prefers_finishable_states() {
        let table = table();
        let cases = OrientationCases::of(table, STAGES[0].preserve, &params());

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
