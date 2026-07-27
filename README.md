# robodoan

Experimental blockbuilding-based search program for a [4D Rubik's cube](https://hypercubing.xyz/puzzles/3x3x3x3/)

This solver completes F2L by blockbuilding and then orients the last cell using algorithms found by search. PLC is not solved yet; see [Last cell](#last-cell).

## Performance

On an M2 Max Macbook Pro, here is how the F2L solver performed on 10 random scrambles of a 4-dimensional 3×3×3×3 Rubik's cube.

These are solutions to F2L, not the whole puzzle.

Move count is approximately STM except for very rare cases where two sequential moves use the same grip, in which case STM would be slightly lower than ETM.

There are two profiles: "fast" and "short."

### Fast profile

```
69 ETM in 8.97840925s
70 ETM in 7.331832625s
67 ETM in 7.840537417s
68 ETM in 7.571879958s
71 ETM in 9.166809125s
68 ETM in 7.999356833s
68 ETM in 7.219549042s
70 ETM in 8.134437291s
69 ETM in 8.066849666s
68 ETM in 8.181445167s

mean movecount: 68.8 ETM
mean time: 8.049 seconds
```

### Short profile

```
66 ETM in 17.397143792s
65 ETM in 17.069599208s
67 ETM in 16.124495667s
64 ETM in 16.10172325s
66 ETM in 19.68221025s
65 ETM in 18.024998208s
67 ETM in 17.205043041s
64 ETM in 15.573559166s
66 ETM in 18.377754833s
64 ETM in 13.521910042s

mean movecount: 65.4 ETM
mean time: 16.908 seconds
```

## Solving strategy

### F2L

#### Stages

F2L is solved in 6 stages of blockbuilding:

| Superstage         | Stage                               | Options | Additional pieces | Initial blocks | Target blocks |
| ------------------ | ----------------------------------- | :-----: | :---------------: | :------------: | :-----------: |
| Mid + left         | 2×2×2×2 (initial)                   |   16    |   2×2×2×2 = 16    |       16       |       1       |
|                    | 2×2×3×2 (extend Y)                  |    4    |    2×2×1×2 = 8    |       9        |       1       |
|                    | 2×3×3×2 (extend Z)                  |    3    |   2×1×3×2 = 12    |       13       |       1       |
| Right (mid + back) | 1×2×2×2 (intial)                    |    8    |    2×2×2×1 = 8    |       9        |       2       |
|                    | 1×2×3×2 (extend Y)                  |    2    |    2×2×1×1 = 4    |       6        |       2       |
| End                | 3×3×3×2 (F2L)                       |    1    |    1×3×1×2 = 2    |       5        |       1       |

- Note 1: The XYZW axes in this table roughly correspond to the XYZW axes of the [3-Block](https://hypercubing.xyz/methods/3x3x3x3/3block/) solving method.
- Note 2: Each stage has multiple options due to the symmetry of the puzzle.

These stages are selected to avoid dead ends while blockbuilding, where the blocks that have already been built get in the way of forming new ones.

#### Search algorithm

To find a solution to a stage, we use [iterative deepening depth-first search](https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search) up to a maximum depth (currently 4) to get to the target block count.

If we cannot reach the target block count, we increase the target block count by 1 and run the search again. If we are unable to find a solution that increases the block count at all, we increase the maximum depth and restart with the original target block count.

After we find a partial solution that solves some number of blocks, we re-run the search starting from the partial solution with the original target block count and maximum depth.

#### Pruning heuristics

The depth-first search rejects branches where the probability of forming enough blocks to meet the target is zero (using `Heuristic::Correct`) or very low (using `Heuristic::Fast`).

In practice, the faster heuristics lead to shorter solutions because even though some possibilities are discounted, we are able to search to a greater depth.

##### Correct heuristics

- **Combinatoric limit:** At most `2^remaining_moves * target_blocks` block pairings can be completed in the remaining moves.
- **Grip-theoretic limit:**
  - For each unordered pair of blocks `(b1, b2)`:
    - Let `m` be the grip along which they will differ when solved.
    - Let `m1 = b1.attitude * m` and `m2 = b2.attitude * m`.
    - Iff `m1 = m2`, then the blocks can be paired in 1 move. Assume that each subsequent move also creates one pairing; add `remaining_moves` to `max_blocks_solvable`.
    - Let the "free grips" be the intersection of the inactive grip sets of `b1` and `b2`.
    - Iff there is a twist of one of the free grips that takes `m1` to `m2` (equivalently: whose inverse takes `m2` to `m1`) then the blocks can be paired in 2 moves. Assume that each subsequent move also creates one pairing; add `remaining_moves - 1` to `max_blocks_solvable`.
    - Otherwise, the block can be paired in 3 moves. Assume that each subsequent move also creates one pairing; add `remaining_moves - 2` to `max_blocks_solvable`.
  - At most `max_blocks_solvable` block pairings can be completed in the remaining moves.

##### Fast heuristics

- **Combinatoric limit:** At most `2^n` block pairings can be completed in `n` moves.
- **Grip-theoretic limit:** Same as correct heuristic.

#### Representation

The puzzle state is represented using a stack-allocated list of blocks with a maximum length determined by a compile-time constant.

Each block is represented using 3 bytes:

- 2 bytes for a layer mask (3 bits × 4 axes)
- 1 byte for the attitude (ID from 0 to 191 inclusive)

Since each block is represented using 3 bytes, we're able to fit a puzzle state containing **21** blocks (63 bytes) + length (1 byte) in exactly 64 bytes.

### Last cell

Once blockbuilding finishes, every piece outside one cell is solved and the
remaining 26 pieces can only be moved by sequences that break F2L and then put
it back. Those sequences are *algorithms*, and the solver finds them ahead of
time rather than searching for them mid-solve.

#### Finding algorithms

Searching directly for F2L-preserving sequences is hopeless: there are 184
twists, so even depth 8 is out of reach. Instead we meet in the middle, using a
key that describes where the F2L pieces are while saying nothing about the last
cell (`PuzzleState::f2l_key`). If two sequences reach states with the same key
they have moved F2L to exactly the same place, so the first followed by the
reverse of the second restores F2L and touches only the last cell. Enumerating
to depth `d` from one side yields algorithms up to `2d` twists long.

Two details make the table much richer for free:

- **Rotations.** Rotating the whole puzzle about the last cell costs nothing, so
  every algorithm found is closed under the 24 rotations that fix that cell. A
  search using only `R` and the last cell therefore produces algorithms on all
  six side grips.
- **Two side grips.** No algorithm using a single side grip can misorient a 2c
  piece, however deep you search — the 4D echo of 3D edge orientation surviving
  `<R, L, U, D>`. So the default table runs two passes: one grip to depth 8, and
  two grips to depth 6.

#### Solving the last cell

The search treats whole algorithms as moves and beam-searches over them. Twists
of the last cell are algorithms too (they cost one move and preserve
everything), so RKT-style setups fall out of the same mechanism instead of
needing to be bolted on.

A human splits OLC into three steps — 2c with EOLL algorithms, then 3c, then 4c
— because each step has a *pure* algorithm touching one piece type. Those
algorithms are 3D algorithms lifted through RKT, so a seven-move 3D sune becomes
thirteen twists. Searching for genuinely 4D algorithms instead turns up much
shorter ones, but they are never pure: **there is no algorithm within reach that
twists 4c pieces while leaving 3c orientation alone**. So this solver orients
every piece type against a single joint objective, which is what a human does
for fewest-moves anyway.

Falling short of a goal still keeps the progress made rather than discarding it,
since an algorithm that orients most of what it touches is usually worth having.

#### Status

Building the default table takes about 25 seconds and yields ~1.7M algorithms.
It is independent of the scramble, so one table serves every solve, and
`AlgTable::load_or_generate` keeps it in `alg_table.bin` between runs — 27 MB,
and 0.65 s to load instead of 26 s to build. Only the twists are stored; the
effect on the last cell and the move count are recomputed on load, which keeps
the file small and means a corrupt one cannot smuggle in an algorithm that
fails to preserve F2L. The file records a fingerprint of the parameters it was
built for and is regenerated if they change.

On six random scrambles with the fast profile, OLC finishes every time and adds
roughly 25 ETM on top of F2L, taking under 30 seconds
(`cargo run --release --example end_to_end`):

```text
94  93  98  95  93  95   ETM total, mean 94.7, all fully oriented
```

#### Ranking states, and the trap in counting pieces

Getting OLC to finish at all turned on how states are ranked. Counting
misoriented pieces and heading downhill *reliably* stopped one piece short,
every single run. The reason is that the count is the wrong measure:
`[0, 1, 0]` has one piece wrong and looks nearly done, but no algorithm clears
it, while `[0, 2, 2]` has four wrong and is often a single algorithm from done.
Piece counting actively prefers the dead end.

So `OrientationCases` ranks by what the algorithms can actually do, at two
resolutions:

1. Exactly one algorithm away. `CellState::orientation_key` packs which way
   every piece's last-cell sticker points. Orientation evolves on its own — an
   algorithm sends the sticker at `j` to `attitude[j] * (sticker at source[j])`,
   which depends on the incoming orientation and nothing else — so two states
   with the same key are oriented by the same algorithms. That makes "am I one
   algorithm from done?" an exact hash lookup, built over the *whole* table
   rather than the subset the beam can afford to branch on.
2. Otherwise, whether the `[ridges, edges, corners]` profile is one some
   algorithm produces at all. Coarser, but it gives the search a gradient to
   follow while it is still too far out for an exact hit.

Two smaller things also help. The beam reserves part of itself for
worse-looking states (`per_distance_cap`), since the endgame needs uphill moves.
And ties are broken by how many of the cell's 24 rows are already built into
bars — a row is a bar when its three pieces share one attitude, so they sit
rigidly together and an algorithm can carry them as a unit. That is
blockbuilding, just inside the last cell.

#### Still unfinished

**PLC.** Permuting the 2c pieces needs a 3-cycle that preserves all orientation.
The shortest ones known use wide moves, which this solver's move set does not
have, and every orientation-preserving algorithm in an 8-twist table permutes
the 2c pieces only as whole-cell rotations. The stage does make partial progress
now that OLC finishes — 2c unsolved counts drop from six to two or three — but it
does not close. A deeper or wide-move-aware table would fix this.

What the solver aims to leave behind is a fully oriented 3×3×3 needing only its
3c and 4c pieces permuted, which is exactly the input an ordinary 3^3 solver
wants, lifted back through RKT.

## Representation

## History

This program is named after Charles Doan, the 3^4 FMC (fewest-moves challenge) world record holder at the time this program was developed. As of September 2024, Charles Doan holds both the computer-assisted and non-computer-assisted FMC records for the 3×3×3×3 puzzle, and in fact his submission for non-computer-assisted is even shorter than the computer-assisted solution.

I'm writing this program to give the computer-assisted category some much-needed love.

## Optimizations (to-do)

- [x] search multiple routes at once / meta search over block extensions
- [x] don't move the same grip twice within one search
- [x] indistinguishable attitudes
- [x] prune based on optimistic block formation heuristics
- [x] dynamically adjust goal based on depth
- [ ] NISS
- [ ] don't search duplicates (only consider pieces for current stage)
