//! Diagnoses why OLC stalls: is the last piece unreachable, or just not found?

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

    for seed in 0..trials {
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);
        let scramble = RUBIKS_4D.random_moves(&mut rng, 100);
        let f2l = Solver::new(Profile::Fast, scramble.clone()).solve();
        let mut state = PuzzleState::default();
        state.do_twists(&scramble);
        state.do_twists(&f2l);

        for (label, params) in [
            (
                "default ",
                LastCellSearchParams {
                    verbosity: 0,
                    ..Default::default()
                },
            ),
            (
                "wide    ",
                LastCellSearchParams {
                    beam_width: 800,
                    max_algs_per_stage: 40_000,
                    max_rounds: 20,
                    verbosity: 0,
                    ..Default::default()
                },
            ),
        ] {
            let t = std::time::Instant::now();
            let s = solve_last_cell(&state, &table, &params);
            println!(
                "#{seed} {label}: {:3} ETM, unoriented {:?}  ({:.1?})",
                s.cost,
                s.residual.unoriented(),
                t.elapsed()
            );

            // Can ANY single algorithm in the whole table finish from here?
            if s.residual.unoriented() != [0, 0, 0] {
                let rescue = table
                    .algs()
                    .iter()
                    .filter(|a| a.effect.apply(s.residual).unoriented() == [0, 0, 0])
                    .min_by_key(|a| a.cost);
                match rescue {
                    Some(a) => println!("      one-alg finish EXISTS in the table: {a}"),
                    None => println!("      NO single algorithm in the table finishes from here"),
                }
            }
        }
    }
}
