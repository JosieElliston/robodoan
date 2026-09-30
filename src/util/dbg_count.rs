use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};

#[linkme::distributed_slice]
pub static DBG_COUNT_NAMES: [&'static str];

type Shard = Arc<[AtomicU64]>;
static SHARDS: Mutex<Vec<Shard>> = Mutex::new(vec![]);
thread_local! {
    static SHARD: Shard = {
        let shard: Shard = DBG_COUNT_NAMES.iter().map(|_| AtomicU64::new(0)).collect();
        SHARDS.lock().unwrap().push(Arc::clone(&shard));
        shard
    };
}

#[inline]
pub fn record(slot: &'static &'static str) {
    let base = DBG_COUNT_NAMES.as_ptr() as usize;
    let index = (slot as *const &str as usize - base) / size_of::<&str>();
    SHARD.with(|shard| {
        let n = &shard[index];
        n.store(n.load(Relaxed) + 1, Relaxed);
    });
}

pub fn dbg_counts() -> HashMap<&'static str, u64> {
    let mut ret = HashMap::new();
    for shard in SHARDS.lock().unwrap().iter() {
        for (name, n) in DBG_COUNT_NAMES.iter().zip(shard.iter()) {
            *ret.entry(*name).or_default() += n.load(Relaxed);
        }
    }
    ret
}
