use std::collections::BTreeSet;
use std::fmt;
use std::ops::Index;

use itertools::Either;

#[cfg(feature = "dbg_rank_counts")]
use std::collections::HashMap;

#[cfg(feature = "dbg_twist_count")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(any(feature = "dbg_rank_counts", feature = "dbg_twist_count"))]
use std::sync::{Arc, Mutex};

use super::{Block, BlockListMeta};
use crate::sim::common::*;
use crate::util::bitset::BitSet32;

/// Maximum number of blocks that can be stored.
const MAX_BLOCK_COUNT: u32 = 26;

// TODO: make this not copy
/// Partial puzzle state, stored as a list of blocks.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct BlockList {
    /// List of blocks.
    blocks: [Block; MAX_BLOCK_COUNT as usize],
    // TODO: should sort by rank and use CSR?
    /// Bitmap indicating, for each possible inner rank value, the indices of
    /// blocks with that inner rank.
    inner_ranks: [BitSet32; 5],
    /// Packed metadata.
    meta: BlockListMeta,
}

/// Assertion of `std::mem::size_of::<BlockList>()`.
///
/// It doesn't matter that much, but it's nice to keep it small if we can.
const _SIZE_ASSERT: [u8; 128] = [0; std::mem::size_of::<BlockList>()];

impl fmt::Debug for BlockList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockList")
            .field("blocks", &self.blocks())
            .field("inner_ranks", &self.inner_ranks)
            .field("meta", &self.meta)
            .finish()
    }
}

impl Index<u32> for BlockList {
    type Output = Block;

    fn index(&self, index: u32) -> &Self::Output {
        debug_assert!(index < self.len());
        &self.blocks[index as usize]
    }
}

impl Index<u8> for BlockList {
    type Output = Block;

    fn index(&self, index: u8) -> &Self::Output {
        &self[index as u32]
    }
}

impl BlockList {
    /// Maximum number of blocks that can be stored
    pub const MAX_LEN: u32 = MAX_BLOCK_COUNT;

    /// Empty block list
    pub const EMPTY: Self = Self {
        blocks: [Block::EMPTY; MAX_BLOCK_COUNT as usize],
        inner_ranks: [BitSet32::EMPTY; 5],
        meta: BlockListMeta::DEFAULT,
    };

    /// Returns the number of blocks.
    pub fn len(&self) -> u32 {
        self.meta.block_count() as u32
    }

    /// Returns whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `Some(self)` if nonempty or `None` if empty.
    pub fn if_nonempty(self) -> Option<Self> {
        (!self.is_empty()).then_some(self)
    }

    /// Returns the list of blocks.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks[..self.len() as usize]
    }

    /// Returns an iterator over blocks with a specific inner rank.
    pub fn blocks_with_inner_rank(&self, inner_rank: u8) -> impl '_ + Iterator<Item = Block> {
        self.inner_ranks[inner_rank as usize]
            .iter()
            .map(|i| self[i])
    }

    /// Returns the twist count.
    pub fn twist_count(&self) -> u16 {
        self.meta.twist_count()
    }

    /// Adds a block to the end of the list and updates the rank bitmaps.
    ///
    /// Returns an error and sets `self.len > MAX_BLOCK_COUNT` in case of
    /// overflow.
    fn push(&mut self, block: Block) -> Result<(), ()> {
        let i = self.len();

        // Update len
        self.meta.increment_block_count();
        if self.len() > MAX_BLOCK_COUNT {
            return Err(());
        }

        // Update ranks
        self.inner_ranks[block.inner_rank() as usize].set_from_0(i as u8);

        // Update blocks
        self.blocks[i as usize] = block;

        Ok(())
    }

    /// Sets the block at the given index.
    ///
    /// The new block must have the same rank as the old block.
    fn set_block_with_same_rank(&mut self, index: u32, new: Block) {
        let old = self.blocks[index as usize];

        debug_assert_ne!(old, new);
        debug_assert_eq!(old.inner_rank(), new.inner_rank());

        self.blocks[index as usize] = new;
    }
    /// Sets a block to empty and updates all assorted fields except `self.len`.
    ///
    /// This leaves the block list in an invalid state which must be cleaned up
    /// using [`Self::cleanup()`].
    fn remove_block(&mut self, index: u32) {
        let old = self.blocks[index as usize];

        // Update ranks
        self.inner_ranks[old.inner_rank() as usize].clear_from_1(index as u8);

        // Update blocks
        self.blocks[index as usize] = Block::EMPTY;
    }

    /// Applies `setup_moves` to each piece in `block` and then adds all the
    /// pieces into the puzzle state, except for the ones that are already in
    /// the puzzle state.
    ///
    /// The attitude of `block` is ignored.
    ///
    /// Returns [`BlockList::EMPTY`] if there are too many blocks.
    #[must_use]
    pub fn add_block_with_setup_moves(&self, setup_moves: &[Twist], block: Block) -> Self {
        let mut ret = self.clone();

        // TODO: extract into function on Block
        let pieces_from_block_at_solved = |block: Block| {
            let mut blocks = vec![block];
            for g in Grip::ALL {
                // TODO: optimize this
                blocks = blocks
                    .into_iter()
                    .flat_map(|b| b.split(g))
                    .filter(|b| !b.is_empty())
                    .collect();
            }
            blocks
                .into_iter()
                .map(|b| Piece::new_solved(b.active_grips().iter()))
        };

        let mut new_pieces = pieces_from_block_at_solved(block).collect::<BTreeSet<Piece>>();
        for i in 0..self.len() {
            for piece in pieces_from_block_at_solved(self[i]) {
                new_pieces.remove(&piece);
            }
        }

        let init_piece = |new_piece| setup_moves.iter().fold(new_piece, |p, &twist| twist * p);

        for piece in new_pieces {
            let transformed_piece = init_piece(piece);
            if ret
                .push(transformed_piece.attitude * Block::from(piece))
                .is_err()
            {
                return Self::EMPTY; // indicate error
            };
        }

        ret.cleanup();

        ret
    }

    /// Applies all `grip.twists()`.
    /// Also gives the applied twist for convenience.
    ///
    /// An element is [`BlockList::EMPTY`] if there are too many blocks for that element.
    pub fn grip_twists(&self, grip: Grip) -> impl Iterator<Item = (Twist, BlockList)> {
        const N: u32 = 16;

        if self.len() > N {
            return Either::Left(self.grip_twists_fallback(grip));
        }

        let mut active_blocks = [Block::EMPTY; N as usize];
        let mut inactive_blocks = [Block::EMPTY; N as usize];

        let mut active_len = 0;
        let mut inactive_len = 0;

        // let mut active_ranks: [BitSet32; 5];

        for i in 0..self.len() {
            let block = self[i];
            let [active, inactive] = block.split(grip);

            if !active.is_empty() {
                #[cfg(debug_assertions)]
                if inactive.is_empty() {
                    debug_assert_eq!(active, block);
                } else {
                    debug_assert_eq!(active.inner_rank(), block.inner_rank() + 1);
                }
                active_blocks[active_len] = active;
                active_len += 1;
            }

            if !inactive.is_empty() {
                debug_assert_eq!(inactive.inner_rank(), block.inner_rank());
                inactive_blocks[inactive_len] = inactive;
                inactive_len += 1;
            }

            #[cfg(debug_assertions)]
            match (active.is_empty(), inactive.is_empty()) {
                (true, true) => debug_assert_eq!(active.inner_rank(), inactive.inner_rank() + 1),
                (true, false) => debug_assert_eq!(inactive.inner_rank(), block.inner_rank()),
                (false, true) => debug_assert_eq!(active.inner_rank(), block.inner_rank()),
                (false, false) => (),
            }
        }

        // {
        //     let s = format!("active_len: {active_len}, inactive_len: {inactive_len}");
        //     dbg_count!(s);
        // }

        let blocks_len = active_len + inactive_len;
        if blocks_len > MAX_BLOCK_COUNT as usize {
            return Either::Right(Either::Left(
                grip.twists().into_iter().map(|twist| (twist, Self::EMPTY)),
            ));
        }

        let mut blocks = [Block::EMPTY; MAX_BLOCK_COUNT as usize];
        blocks[..active_len].copy_from_slice(&active_blocks[..active_len]);
        blocks[active_len..blocks_len].copy_from_slice(&inactive_blocks[..inactive_len]);

        let mut inner_ranks = [BitSet32::EMPTY; 5];
        for i in 0..blocks_len {
            // TODO: perf
            inner_ranks[blocks[i].inner_rank() as usize].set_from_0(i as u8);
        }

        let mut meta = self.meta;
        meta.count_twist_on_grip(grip);
        meta.set_block_count(blocks_len as u8);

        let split = Self {
            blocks,
            inner_ranks,
            meta,
        };

        let twisted = grip.twists().into_iter().map(move |twist| {
            #[cfg(feature = "dbg_twist_count")]
            record_twist();

            let mut ret = split;
            for i in 0..active_len {
                ret.blocks[i] = twist.transform * ret.blocks[i];
            }
            ret.cleanup();
            
            (twist, ret)
        });
        Either::Right(Either::Right(twisted))
    }

    /// Applies all `grip.twists()`.
    /// Also gives the applied twist for convenience.
    ///
    /// An element is [`BlockList::EMPTY`] if there are too many blocks for that element.
    fn grip_twists_fallback(&self, grip: Grip) -> impl Iterator<Item = (Twist, BlockList)> {
        grip.twists()
            .into_iter()
            .map(|twist| (twist, self.twist(twist)))
    }

    /// Applies a twist.
    ///
    /// Returns [`BlockList::EMPTY`] if there are too many blocks.
    #[must_use]
    pub fn twist(&self, twist: Twist) -> BlockList {
        debug_assert_ne!(twist.transform, Elem::IDENT);

        #[cfg(feature = "dbg_twist_count")]
        record_twist();

        let mut ret = self.clone();

        ret.meta.count_twist_on_grip(twist.grip);

        // Split blocks and apply twist.
        for i in 0..ret.len() {
            let block = ret[i];
            let [active, inactive] = block.split(twist.grip);
            if active.is_empty() {
                continue; // no change
            } else {
                let active = twist.transform * active;
                if inactive.is_empty() {
                    ret.set_block_with_same_rank(i, active); // all active
                } else {
                    // TODO: use this fact to not compute inactive.inner_rank when pushing
                    debug_assert_eq!(active.inner_rank(), inactive.inner_rank() + 1);
                    ret.set_block_with_same_rank(i, inactive); // inactive has same inner rank
                    if ret.push(active).is_err() {
                        return Self::EMPTY; // indicate error
                    }
                }
            }
        }

        ret.cleanup();

        ret
    }

    /// Merges blocks and sorts them, canonicalizing the whole list.
    ///
    /// This is idempotent.
    fn cleanup(&mut self) {
        self.cleanup_inner();

        #[cfg(debug_assertions)]
        {
            // Check that `cleanup_inner()` is idempotent.
            let old = self.clone();
            self.cleanup_inner();
            assert_eq!(*self, old, "cleanup() is not idempotent");
        }
    }

    fn cleanup_inner(&mut self) {
        self.merge_blocks_until();

        // Filter out empty blocks
        let mut len = self.len() as usize;
        let mut i = 0;
        while i < len {
            while self.blocks[i].is_empty() && i < len {
                len -= 1;
                self.blocks.swap(i, len);
            }
            i += 1;
        }
        self.meta.set_block_count(len as u8);

        // Canonicalize block order.
        self.blocks[..len].sort();

        // Update inner ranks
        self.inner_ranks = Default::default();
        for (i, &b) in self.blocks[..len].iter().enumerate() {
            self.inner_ranks[b.inner_rank() as usize].set_from_0(i as u8);
        }
    }

    /// Merge blocks until we reach a fixed point.
    fn merge_blocks_until(&mut self) {
        if self.len() > 16 {
            // this never happens in practice
            while self.merge_blocks_once_fallback() {}
            return;
        }

        while self.merge_blocks_once() {}
    }

    /// Merges all blocks that can be merged.
    ///
    /// Returns whether any blocks were merged.
    fn merge_blocks_once(&mut self) -> bool {
        #[cfg(feature = "dbg_rank_counts")]
        record_rank_counts(&self.inner_ranks);

        let mut any_merged = false;

        // It's important to iterate from largest to smallest rank, so that we
        // prioritize blocks where attitudes must match exactly (like
        // corner+edge) instead of blocks where many attitudes are
        // indistinguishable (like center+core).
        for body_rank in (0..4usize).rev() {
            let head_rank = body_rank + 1;
            let body_candidates = self.inner_ranks[body_rank as usize].clone();
            for body_index in body_candidates {
                // TODO: bc we sorted them,
                // i think we have that the bodies of given rank are contiguous
                // actually no, we sorted them lexicographically, not by rank
                // check if rank then lexicographical tiebreaks so they're contiguous is faster
                let body = self[body_index];

                let head_candidates = self.inner_ranks[head_rank as usize].clone();
                'loop_per_head: for head_index in head_candidates {
                    #[cfg(false)]
                    {
                        let head = self[head_index];
                        let merged = Block::merge(body, head);
                        if merged.is_empty() {
                            dbg_count!("oracle not_mergeable");
                        } else {
                            dbg_count!("oracle mergeable");
                        }

                        let head = self[head_index];
                        if !Block::dbg_can_merge_layers(body, head) {
                            dbg_count!("layers not_mergeable");
                        } else {
                            dbg_count!("layers mergeable");
                        }
                    }

                    let head = self[head_index];

                    let merged = Block::merge(body, head);

                    if !merged.is_empty() {
                        any_merged = true;
                        // Remove head
                        self.remove_block(head_index as u32);
                        // Replace body (inner rank stays the same)
                        self.set_block_with_same_rank(body_index as u32, merged);

                        break 'loop_per_head;
                    }
                }
            }
        }

        any_merged
    }

    /// Merges all blocks that can be merged.
    ///
    /// Returns whether any blocks were merged.
    fn merge_blocks_once_fallback(&mut self) -> bool {
        let mut any_merged = false;

        // It's important to iterate from largest to smallest rank, so that we
        // prioritize blocks where attitudes must match exactly (like
        // corner+edge) instead of blocks where many attitudes are
        // indistinguishable (like center+core).
        for body_rank in (0..4).rev() {
            let head_rank = body_rank + 1;
            let body_candidates = self.inner_ranks[body_rank as usize].clone();
            for body_index in body_candidates {
                let body = self[body_index];

                let head_candidates = self.inner_ranks[head_rank as usize].clone();
                'loop_per_head: for head_index in head_candidates {
                    let head = self[head_index];

                    let merged = Block::merge(body, head);

                    if !merged.is_empty() {
                        any_merged = true;
                        // Remove head
                        self.remove_block(head_index as u32);
                        // Replace body (inner rank stays the same)
                        self.set_block_with_same_rank(body_index as u32, merged);

                        break 'loop_per_head;
                    }
                }
            }
        }

        any_merged
    }
}

impl FromIterator<Block> for BlockList {
    fn from_iter<T: IntoIterator<Item = Block>>(iter: T) -> Self {
        let mut ret = Self::EMPTY;
        for block in iter {
            if ret.push(block).is_err() {
                return Self::EMPTY;
            }
        }
        ret.cleanup();
        ret
    }
}

#[cfg(feature = "dbg_twist_count")]
static TWIST_COUNTS: Mutex<Vec<Arc<AtomicU64>>> = Mutex::new(vec![]);
#[cfg(feature = "dbg_twist_count")]
thread_local! {
    static TWIST_COUNT_SHARD: Arc<AtomicU64> = {
        let shard = Arc::<AtomicU64>::default();
        TWIST_COUNTS.lock().unwrap().push(Arc::clone(&shard));
        shard
    };
}

#[cfg(feature = "dbg_twist_count")]
fn record_twist() {
    // only this thread writes its shard, so no need for an atomic RMW
    TWIST_COUNT_SHARD
        .with(|shard| shard.store(shard.load(Ordering::Relaxed) + 1, Ordering::Relaxed));
}

#[cfg(feature = "dbg_twist_count")]
pub fn twist_count() -> u64 {
    TWIST_COUNTS
        .lock()
        .unwrap()
        .iter()
        .map(|shard| shard.load(Ordering::Relaxed))
        .sum()
}

/// counters of each `inner_ranks` popcount profile
#[cfg(feature = "dbg_rank_counts")]
type RankCountShard = Arc<Mutex<HashMap<[u8; 5], u64>>>;
#[cfg(feature = "dbg_rank_counts")]
static RANK_COUNTS: Mutex<Vec<RankCountShard>> = Mutex::new(vec![]);
#[cfg(feature = "dbg_rank_counts")]
thread_local! {
    static RANK_COUNT_SHARD: RankCountShard = {
        let shard = RankCountShard::default();
        RANK_COUNTS.lock().unwrap().push(Arc::clone(&shard));
        shard
    };
}

#[cfg(feature = "dbg_rank_counts")]
fn record_rank_counts(inner_ranks: &[BitSet32; 5]) {
    let key = inner_ranks.map(|r| r.count_ones() as u8);
    RANK_COUNT_SHARD.with(|shard| *shard.lock().unwrap().entry(key).or_default() += 1);
}

#[cfg(feature = "dbg_rank_counts")]
pub fn rank_counts() -> HashMap<[u8; 5], u64> {
    let mut ret = HashMap::new();
    for shard in RANK_COUNTS.lock().unwrap().iter() {
        for (&k, &n) in shard.lock().unwrap().iter() {
            *ret.entry(k).or_default() += n;
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_block() {
        let init_block = Block::from_layer_bits(0x0FF);
        let state = BlockList::default().add_block_with_setup_moves(&[], init_block);
        assert_eq!(state.len(), 1);
        assert_eq!(state[0_u32], init_block);

        let ru = Twist::new(grips::R, elements::WZ);
        let ir = Twist::new(grips::I, elements::ZY);
        let state = BlockList::default().add_block_with_setup_moves(&[ru, ir], init_block);
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn test_merge_2223() {
        let head = Block::from_layer_bits(0xc73);
        let body = Block::from_layer_bits(0x4fb);
        dbg!(head, body);
        let mut list = BlockList::default();
        list.push(body).unwrap();
        list.push(head).unwrap();
        list.cleanup();
        dbg!(Block::merge(body, head));
        assert_eq!(list.len(), 1);
    }
}
