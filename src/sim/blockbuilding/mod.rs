mod block;
mod layer_mask;
mod piece;
mod state;

use std::sync::atomic::AtomicU64;

pub use block::*;
pub use layer_mask::*;
pub use piece::*;
pub use state::*;

pub static COUNTER_A: AtomicU64 = AtomicU64::new(0);
pub static COUNTER_B: AtomicU64 = AtomicU64::new(0);
pub static COUNTER_C: AtomicU64 = AtomicU64::new(0);
pub static COUNTER_D: AtomicU64 = AtomicU64::new(0);
