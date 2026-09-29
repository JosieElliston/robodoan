use std::error::Error;

use itertools::Itertools;
use rand::SeedableRng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use robodoan::{sim::ALL_TWISTS, *};

fn main() -> Result<(), Box<dyn Error>> {
    let params = BlockBuildingSearchParams::default();

    if let Some(filename) = std::env::args().nth(1) {
        let log_file_text = std::fs::read_to_string(&filename)?;
        let scramble: mc4d::Mc4dScramble = log_file_text.parse()?;
        println!("Loaded log file from {filename}");
        println!();
        // let (solve_twists, _elapsed_time) = search_4d(scramble.scramble());
        let solve_twists = robodoan::Solver::new(params, scramble.scramble()).solve();
        println!();
        std::fs::write("out.log", scramble.to_string(false, solve_twists))?;
        return Ok(());
    }

    let mut results = vec![];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(123);
    for i in 0..10 {
        let scramble = sim::random_twists(&mut rng, 100);
        println!("\n\n---- STARTING SEARCH #{} ----\n", i + 1);
        println!("Scramble: {}", scramble.iter().join(" "));
        let t = std::time::Instant::now();
        let solution = robodoan::Solver::new(params, scramble).solve();
        results.push((solution.len(), t.elapsed()));
    }
    println!("\n\n---- RESULTS ----\n");
    for (move_count, time) in results {
        println!("{move_count} ETM in {time:?}");
    }

    #[cfg(feature = "dbg_rank_counts")]
    print_rank_counts();

    return Ok(());

    println!();
    println!();
    let mut examples = gpu_test::take_all_examples();
    examples.truncate(1024 * 64);

    let states = examples.iter().map(|(b, _)| b.clone()).collect_vec();
    let twists = &*ALL_TWISTS;

    println!(
        "Testing {} states * {} twists on GPU",
        states.len(),
        twists.len(),
    );

    println!("Executing on CPU ...");
    let t = std::time::Instant::now();
    let expected: Vec<_> = states
        .par_iter()
        .flat_map_iter(|block_list| twists.iter().map(|&twist| block_list.twist(twist).len()))
        .collect();
    println!("Done in {:?}!", t.elapsed());

    let mut gpu = robodoan::gpu::Gpu::new();
    println!("Executing on GPU ...");
    let t = std::time::Instant::now();
    let actual = gpu.test_do_twist(&states, &twists);

    // let mut times = vec![];
    // for _ in 0..20 {
    //     let t1 = std::time::Instant::now();
    //     gpu.test_do_twist(&states, &twists);
    //     times.push(t1.elapsed());
    // }
    // let len = times.len() as f64;
    // let ms = times.iter().map(|d| d.as_secs_f64() * 1000.0);
    // let avg = ms.clone().sum::<f64>() / len;
    // let stddev = (ms.map(|s| (s - avg) * (s - avg)).sum::<f64>() / len).sqrt();
    // println!("average: {avg} ms, stddev: {stddev} ms");

    println!("Done in {:?}! Checking results ...", t.elapsed());

    assert_eq!(expected.len(), actual.len());

    for (i, (exp, act)) in itertools::izip!(&expected, actual).enumerate() {
        if *exp != act {
            println!("failed on index {i}");
            dbg!(states[i / 184]);
            dbg!(twists[i % 184]);
            dbg!(i);
            dbg!(i / 184);
            dbg!(i % 184);
            pretty_assertions::assert_eq!(*exp, act);
        }
    }
    println!("SUCCESS! They all matched!");

    Ok(())
}

#[cfg(feature = "dbg_rank_counts")]
fn print_rank_counts() {
    let mut counts = sim::blockbuilding::rank_counts().into_iter().collect_vec();
    counts.sort_by_key(|&(k, n)| (std::cmp::Reverse(n), k));
    let total: u64 = counts.iter().map(|&(_, n)| n).sum();

    println!("\n\n---- INNER RANK POPCOUNTS ----\n");
    println!(
        "{} merge_blocks() calls, {} distinct profiles",
        total,
        counts.len()
    );

    // Per-rank marginals: mean and max popcount for each inner rank.
    println!("\nrank   mean    max");
    for r in 0..5 {
        let sum: u64 = counts.iter().map(|&(k, n)| k[r] as u64 * n).sum();
        let max = counts.iter().map(|&(k, _)| k[r]).max().unwrap_or(0);
        println!(
            "{r:>4} {:>6.2} {}",
            sum as f64 / total.max(1) as f64,
            heat(max as u32, 6, 6),
        );
    }

    // Histogram of total block count.
    println!("\nblocks      count       %    cum%");
    let mut cum = 0;
    for (len, group) in &counts
        .iter()
        .map(|&(k, n)| (profile_len(k), n))
        .sorted()
        .chunk_by(|&(len, _)| len)
    {
        let n: u64 = group.map(|(_, n)| n).sum();
        cum += n;
        println!(
            "{} {n:>10} {:>6.2}% {:>6.2}%",
            heat(len, 16, 6),
            100.0 * n as f64 / total as f64,
            100.0 * cum as f64 / total as f64,
        );
    }

    // Most common profiles.
    println!("\n r0 r1 r2 r3 r4  blocks        count       %    cum%");
    let mut cum = 0;
    for &(k, n) in counts.iter().take(30) {
        cum += n;
        let cells = k.iter().map(|&x| heat(x as u32, 6, 3)).join("");
        println!(
            "{cells}  {} {n:>12} {:>6.2}% {:>6.2}%",
            heat(profile_len(k), 16, 6),
            100.0 * n as f64 / total as f64,
            100.0 * cum as f64 / total as f64,
        );
    }

    /// Total number of blocks in an inner rank profile.
    fn profile_len(k: [u8; 5]) -> u32 {
        k.iter().map(|&x| x as u32).sum()
    }

    /// Right-aligns `value` to `width` and colors it on a cold-to-hot scale, where
    /// `max` gets the hottest color. Padding is applied before the escape codes so
    /// columns stay aligned.
    fn heat(value: u32, max: u32, width: usize) -> String {
        // 256-color palette: gray, blue, cyan, green, yellow, orange, red, magenta
        const PALETTE: [u8; 8] = [240, 33, 44, 40, 226, 208, 196, 201];
        let i = if value == 0 {
            0
        } else {
            1 + ((value - 1) * (PALETTE.len() as u32 - 1) / max.max(1))
                .min(PALETTE.len() as u32 - 2)
        };
        format!("\x1b[38;5;{}m{value:>width$}\x1b[0m", PALETTE[i as usize])
    }
}

/// Sets the thread count for the global thread pool.
pub fn set_thread_count(thread_count: usize) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()
        .unwrap();
}
