//! Full solves: blockbuilding for F2L, then the last-cell search.

use itertools::Itertools;
use rand::prelude::*;
use robodoan::lastcell::*;
use robodoan::*;

fn main() {
    let trials: u64 = std::env::args().nth(1).map_or(3, |s| s.parse().unwrap());
    let table = AlgTable::load_or_generate(
        DEFAULT_CACHE_PATH,
        &AlgTableParams {
            verbosity: 1,
            ..Default::default()
        },
    );

    let mut rows = vec![];
    for seed in 0..trials {
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);
        let scramble = RUBIKS_4D.random_moves(&mut rng, 100);
        println!("\n---- scramble #{seed} ----");
        let t = std::time::Instant::now();
        let solution =
            Solver::new(Profile::Fast, scramble.clone()).solve_with_last_cell(Some(&table));

        let mut state = PuzzleState::default();
        state.do_twists(&scramble);
        state.do_twists(&solution);
        let frame = find_canonical_frame(&state);
        let residual = frame.map(|e| CellState::of(&state.reorient(e)));
        rows.push((
            twist_count(&solution),
            t.elapsed(),
            residual.map(|c| c.unoriented()),
            residual.map(|c| c.unsolved()),
        ));
    }

    println!("\n---- RESULTS ----");
    for (cost, time, unoriented, unsolved) in &rows {
        println!("{cost:3} ETM in {time:8.1?}  unoriented {unoriented:?}  unsolved {unsolved:?}");
    }
    println!(
        "mean {:.1} ETM",
        rows.iter().map(|r| r.0).sum::<usize>() as f64 / rows.len() as f64
    );
    let _ = &rows.iter().map(|r| r.1).collect_vec();
}
