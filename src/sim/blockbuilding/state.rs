use std::collections::HashSet;
use std::fmt;
use std::simd::prelude::*;
use std::simd::{LaneCount, SupportedLaneCount};
use std::sync::atomic::AtomicU64;

use itertools::Itertools;

use crate::StackVec;
use crate::sim::*;

// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-tools-preview
// cargo pgo

// pub static LEN_BEFORE: [AtomicU64; 32] = [const { AtomicU64::new(0) }; _];
// pub static LEN_AFTER: [AtomicU64; 32] = [const { AtomicU64::new(0) }; _];
pub static LEN_AFTER: [AtomicU64; 1296] = [const { AtomicU64::new(0) }; _]; // 6^4
pub static LEN_BEFORE: [AtomicU64; 1296] = [const { AtomicU64::new(0) }; _];

/// (64 bytes) State of a puzzle, tracked using [`crate::MAX_BLOCKS`] blocks.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(align(64))]
pub struct BlockSet {
    pub blocks: StackVec<Block, { crate::MAX_BLOCKS }>,
}
impl BlockSet {
    /// Assertion that `std::mem::size_of::<Self>() == 64`.
    ///
    /// It doesn't matter that much, but it's nice to keep it small if we can.
    #[expect(unused)]
    const SIZE_ASSERT: [u8; 64] = [0; std::mem::size_of::<Self>()];

    fn sorted(self) -> Self {
        Self {
            blocks: self.blocks.sorted_unstable(),
        }
    }

    /// Applies a twist and returns the new state.
    ///
    /// Returns `None` if the twist failed because there are too many blocks to
    /// track.
    #[must_use]
    #[inline(never)]
    pub fn old_do_twist(self, twist: Twist, _ndim: usize) -> Option<Self> {
        // panic!("should be unused");
        // LEN_BEFORE[self.blocks.len()].fetch_add(1,
        // std::sync::atomic::Ordering::Relaxed);

        // let premerged = self.old_premerged(twist)?;
        let premerged = self.new_premerged(twist)?;

        // let oracle = premerged.old_merge_blocks(ndim);
        // assert_eq!(
        //     premerged.fully_split().iter().sorted().collect::<Vec<_>>(),
        //     oracle.fully_split().iter().sorted().collect::<Vec<_>>()
        // );
        // assert!(premerged.piece_equivalent(oracle));
        // premerged.assert_piece_equivalent(oracle);
        // let actual = premerged.old_merge_blocks(4);
        let actual = premerged.merge_blocks(4);

        // let actual = premerged.old_merge_blocks(4);

        // if premerged.can_maybe_merge_blocks_len_lte_16() {
        //     LEN_AFTER[(actual.blocks.len() < premerged.blocks.len()) as usize]
        //         .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // }

        // LEN_BEFORE[self.blocks.len()].fetch_add(1,
        // std::sync::atomic::Ordering::Relaxed); LEN_AFTER[premerged.blocks.
        // len() - actual.blocks.len()]     .fetch_add(1,
        // std::sync::atomic::Ordering::Relaxed); let actual =
        // {
        //     let merged_again = actual.old_merge_blocks(4);
        //     assert_eq!(merged_again.blocks.len(), actual.blocks.len());
        // }

        // let mut actual = premerged;
        // actual.merge_blocks_along_axis(twist.grip.axis());

        // assert_eq!(oracle.blocks.len(), actual.blocks.len());
        // assert_eq!(oracle.fully_split(), actual.fully_split());
        // LEN_AFTER[actual.blocks.len()].fetch_add(1,
        // std::sync::atomic::Ordering::Relaxed); Some(oracle)
        // for block in self.blocks {
        //     for axis in 0..4 {
        //         // assert_ne!(block.layers().bits_for_axis(axis), 0b000);
        //         // assert_ne!(block.layers().bits_for_axis(axis), 0b101);
        //         LEN_AFTER[block.layers().bits_for_axis(axis) as usize]
        //             .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //     }
        // }
        // note: pairwise xor popcnt is always >= 2
        // note: pairwise xor popcnt is always 2 or 3 if you can merge them
        // for (i, lhs) in premerged.blocks.iter().enumerate() {
        //     for rhs in premerged.blocks.iter().skip(i + 1) {
        //         let ones = (lhs.layers().to_u16() ^ rhs.layers().to_u16()).count_ones() as usize;
        //         // match lhs.try_merge(*rhs, 4) {
        //         //     Some(_) => {
        //         //         LEN_AFTER[ones].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //         //     }
        //         //     None => {
        //         //         LEN_BEFORE[ones].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //         //     }
        //         // }
        //         // note: conditional on popcnt <= 3, we can merge 0.01226 of the time
        //         if ones <= 3 {
        //             LEN_AFTER[lhs.try_merge(*rhs, 4).is_some() as usize]
        //                 .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //         }
        //     }
        // }

        // for block in self.blocks {
        //     LEN_BEFORE[hash_of_layer(block.layers().to_u16()) as usize]
        //         .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // }
        // for block in premerged.blocks {
        //     LEN_AFTER[hash_of_layer(block.layers().to_u16()) as usize]
        //         .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // }

        Some(actual)
    }
    #[must_use]
    #[inline(never)]
    pub fn do_twist(self, twist: Twist, ndim: usize) -> Option<Self> {
        let ndim = 4;
        return self.old_do_twist(twist, ndim);
        // StackVec::<Block, { crate::MAX_BLOCKS }>
        // let (inside, outsize) = self
        //     .blocks
        //     .into_iter()
        //     .map(|block| {
        //         let [inside, outside] = twist * block;
        //         (inside, outside)
        //     })
        //     .unzip::<
        //         Option<Block>,
        //         Option<Block>,
        //         StackVec<Option<Block>, { crate::MAX_BLOCKS }>,
        //         StackVec<Option<Block>, { crate::MAX_BLOCKS }>,
        //     >()
        //     .map(|(inside, outside)| (inside.into_iter().flatten(),
        // outside.into_iter().flatten()));
        let (inside, outside) = {
            let mut inside = StackVec::<Block, { crate::MAX_BLOCKS }>::new();
            let mut outside = StackVec::<Block, { crate::MAX_BLOCKS }>::new();
            for block in self.blocks {
                let [in_block, out_block] = twist * block;
                if let Some(b) = in_block {
                    inside = inside.push(b).unwrap();
                }
                if let Some(b) = out_block {
                    outside = outside.push(b).unwrap();
                }
            }
            (inside, outside)
        };
        // return Some(
        //     BlockSet {
        //         blocks: outside.extend_from_stackvec(&inside)?,
        //     }
        //     .merge_blocks(ndim),
        // );
        // LEN_BEFORE[inside.len()].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // LEN_AFTER[outside.len()].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let ret = Self::merge_merged_blocks_along_axis(outside, inside, twist.grip.axis());
        // LEN_AFTER[ret.blocks.len()].fetch_add(1,
        // std::sync::atomic::Ordering::Relaxed);
        Some(ret)
        // println!();
        // {
        //     let inside_merged = BlockSet { blocks: inside
        // }.merge_blocks(ndim).blocks;     // println!();
        //     // dbg!(&inside);
        //     // dbg!(&inside_merged);
        //     // assert_eq!(inside_merged.len(), inside.len());
        //     if HashSet::<_>::from_iter(inside_merged) ==
        // HashSet::from_iter(inside) {         COUNTER_A.fetch_add(1,
        // std::sync::atomic::Ordering::Relaxed);     } else {
        //         COUNTER_B.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //     }
        // }
        // {
        //     let outside_merged = BlockSet { blocks: outside
        // }.merge_blocks(ndim).blocks;     // println!();
        //     // dbg!(&outside);
        //     // dbg!(&outside_merged);
        //     // assert_eq!(outside_merged.len(), outside.len());
        //     if HashSet::<_>::from_iter(outside_merged) ==
        // HashSet::from_iter(outside) {         COUNTER_C.fetch_add(1,
        // std::sync::atomic::Ordering::Relaxed);     } else {
        //         COUNTER_D.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //     }
        // }
    }

    fn old_premerged(self, twist: Twist) -> Option<Self> {
        Some(Self {
            blocks: StackVec::<Block, { crate::MAX_BLOCKS }>::from_iter(
                self.blocks
                    .into_iter()
                    .flat_map(|block| twist * block)
                    .flatten(), // Option<T> -> T
            )?,
        })
    }

    fn new_premerged(mut self, twist: Twist) -> Option<Self> {
        let len = self.blocks.len();
        for i in 0..len {
            let block = self.blocks[i];
            let [inside, outside] = twist * block;
            // let [inside, outside] = block.split(twist.grip);
            // let inside = inside.map(|b| Block {
            //     layers: twist.transform * b.layers,
            //     attitude: twist.transform * b.attitude,
            // });
            // if let Some(inside) = inside {
            //     new_blocks = new_blocks.push(inside).unwrap();
            // }
            // if let Some(outside) = outside {
            //     new_blocks = new_blocks.push(outside).unwrap();
            // }
            match (inside, outside) {
                (Some(inside), Some(outside)) => {
                    self.blocks[i] = inside;
                    self.blocks = self.blocks.push(outside).unwrap();
                }
                (Some(inside), None) => {
                    self.blocks[i] = inside;
                }
                (None, Some(outside)) => {
                    self.blocks[i] = outside;
                }
                (None, None) => {
                    unreachable!()
                }
            }
        }
        Some(self)
    }

    // it's ok to have gaps where things are 0
    // really it's a [Option<NonZero<u16>>; 16]
    // TODO: would be nice if the invalid `ElemId`s were ElemId::(u8::MAX)
    fn do_twist_premerged(&self) -> ([u16; 16], [ElemId; 16]) {
        todo!()
        // self.blocks
        //     .into_iter()
        //     .flat_map(|block| twist * block)
        //     .flatten()
    }

    fn any_blocks_merge(self) -> bool {
        for (i, b1) in self.blocks.iter().enumerate() {
            for b2 in self.blocks.iter().skip(i + 1) {
                if (b1.try_merge(*b2, 4)).is_some() {
                    return true;
                }
            }
        }
        false
    }
    fn any_blocks_merge_between(self, other: Self) -> bool {
        for b1 in self.blocks.iter() {
            for b2 in other.blocks.iter() {
                if (b1.try_merge(*b2, 4)).is_some() {
                    return true;
                }
            }
        }
        false
    }

    #[must_use]
    #[inline(never)]
    fn old_merge_blocks(mut self, _ndim: usize) -> Self {
        let ndim = 4;
        for loop_i in 0.. {
            let mut merged_blocks = StackVec::new();
            'b1: for b1 in self.blocks {
                for b2 in &mut merged_blocks {
                    if let Some(merged) = b1.try_merge(*b2, ndim) {
                        *b2 = merged; // replace with merged block
                        continue 'b1;
                    }
                }
                merged_blocks = merged_blocks.push(b1).unwrap();
            }
            if self.blocks.len() == merged_blocks.len() {
                // LEN_AFTER[loop_i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return self;
            }
            self.blocks = merged_blocks;
        }
        unreachable!()
    }
    // #[must_use]
    // #[inline(never)]
    // fn merge_blocks(self, _ndim: usize) -> Self {
    //     // maintains the invariant that merged_blocks cannot be merged any
    // further     let mut merged_blocks: StackVec<Option<Block>, {
    // crate::MAX_BLOCKS }> = StackVec::new();     let mut empty_indexes:
    // StackVec<usize, { crate::MAX_BLOCKS }> = StackVec::new();     'outer: for
    // b1 in self.blocks {         for (outer_i, b2) in
    // merged_blocks.iter().enumerate() {             let Some(b2) = b2 else {
    // continue };             if let Some(mut outer_merged) = b1.try_merge(*b2,
    // 4) {                 merged_blocks[outer_i] = None; empty_indexes =
    //    empty_indexes.push(outer_i).unwrap(); // see if outer_merged can merge
    //    with any already merged
    // block                 'inner: loop {
    //                     for (inner_i, b3) in merged_blocks.iter().enumerate() {
    //                         let Some(b3) = b3 else { continue };
    //                         if let Some(inner_merged) =
    // b3.try_merge(outer_merged, 4) {
    // merged_blocks[inner_i] = None;                             empty_indexes
    // = empty_indexes.push(inner_i).unwrap();
    // outer_merged = inner_merged;                             continue 'inner;
    //                         }
    //                     }
    //                     if let Some(i) = empty_indexes.pop() {
    //                         assert!(merged_blocks[i].is_none());
    //                         merged_blocks[i] = Some(outer_merged);
    //                     } else {
    //                         merged_blocks =
    // merged_blocks.push(Some(outer_merged)).unwrap();                     }
    //                     break;
    //                 }
    //                 continue 'outer;
    //             }
    //         }
    //         merged_blocks = merged_blocks.push(Some(b1)).unwrap();
    //     }
    //     Self {
    //         blocks:
    // StackVec::from_iter(merged_blocks.iter().flatten().copied()).unwrap(),
    //     }
    // }

    #[inline(never)]
    fn old_merge_stackvec_blocks_along_axis<const N: usize>(
        mut blocks: StackVec<Block, N>,
        axis: usize,
    ) -> StackVec<Block, N> {
        if blocks.len() <= 1 {
            return blocks;
        }
        for loop_i in 0.. {
            let mut merged_blocks = StackVec::new();
            'b1: for b1 in blocks {
                for b2 in &mut merged_blocks {
                    if let Some(merged) = b1.try_merge_along_axis(*b2, axis) {
                        *b2 = merged; // replace with merged block
                        continue 'b1;
                    }
                }
                merged_blocks = merged_blocks.push(b1).unwrap();
            }
            if blocks.len() == merged_blocks.len() {
                // LEN_AFTER[loop_i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return blocks;
            }
            blocks = merged_blocks;
        }
        unreachable!()
    }
    #[must_use]
    #[inline(never)]
    fn old_merge_blocks_along_axis(mut self, axis: usize) -> Self {
        Self {
            blocks: Self::old_merge_stackvec_blocks_along_axis(self.blocks, axis),
        }
    }
    #[inline(never)]
    fn merge_merged_blocks_along_axis(
        mut left: StackVec<Block, { crate::MAX_BLOCKS }>,
        right: StackVec<Block, { crate::MAX_BLOCKS }>,
        axis: usize,
    ) -> Self {
        'r: for r in right {
            for l in &mut left {
                if let Some(merged) = l.try_merge_along_axis(r, axis) {
                    *l = merged;
                    continue 'r;
                }
            }
            // not any merges
            left = left.push(r).unwrap();
        }
        // BlockSet { blocks: left }.old_merge_blocks(ndim)
        BlockSet { blocks: left }
    }
    #[inline(never)]
    fn merge_adjacent_blocks(self, ndim: usize) -> Self {
        let mut ret = StackVec::new();
        let mut i = 0;
        while i + 1 < self.blocks.len() {
            if let Some(merged) = self.blocks[i].try_merge(self.blocks[i + 1], ndim) {
                ret = ret.push(merged).unwrap();
                i += 2;
            } else {
                ret = ret.push(self.blocks[i]).unwrap();
                i += 1;
            }
        }
        if i + 1 == self.blocks.len() {
            ret = ret.push(self.blocks[i]).unwrap();
        }
        BlockSet { blocks: ret }
    }
    // fn merge_adjacent_blocks(self, ndim: usize) -> Self {
    //     let mut ret = StackVec::new();
    //     for block in self.blocks {
    //         ret = ret.push(block).unwrap();
    //         // merge blocks from the back
    //         while ret.len() >= 2 {
    //             let right = ret.pop().unwrap();
    //             let left = ret.pop().unwrap();
    //             if let Some(merged) = left.try_merge(right, ndim) {
    //                 ret = ret.push(merged).unwrap();
    //             } else {
    //                 ret = ret.push(left).unwrap();
    //                 ret = ret.push(right).unwrap();
    //                 break;
    //             }
    //         }
    //     }
    //     Self { blocks: ret }
    // }
    // fn merge_blocks(self, ndim: usize) -> Self {
    //     // self.old_merge_blocks(ndim)
    //     self.sorted()
    //         .merge_adjacent_blocks(ndim)
    //         .old_merge_blocks(ndim)
    // }
    // fn merge_blocks(self, ndim: usize) -> Self {
    //     // put blocks into buckets of [0x0000..0x0010, 0x0010..0x0100,
    // 0x0100..0x1000, 0x1000..0xFFFF]     // except these bucket bounds aren't
    // tight     // so maybe [0x0000..0x0008, 0x0010..0x0080, 0x0100..0x0800,
    // 0x1000..0x8000]     let mut buckets: [StackVec<Block, { crate::MAX_BLOCKS
    // }>; 4] = [         StackVec::new(),
    //         StackVec::new(),
    //         StackVec::new(),
    //         StackVec::new(),
    //     ];
    //     // actually this does nothing
    //     for block in self.blocks {
    //         match block.layers().to_u16() {
    //             0x0000..0x0008 => buckets[0] = buckets[0].push(block).unwrap(),
    //             0x0010..0x0080 => buckets[1] = buckets[1].push(block).unwrap(),
    //             0x0100..0x0800 => buckets[2] = buckets[2].push(block).unwrap(),
    //             0x1000..0x8000 => buckets[3] = buckets[3].push(block).unwrap(),
    //             _ => unreachable!("invalid bit pattern for {}", block.layers()),
    //         }
    //     }
    //     // for (i, left) in buckets.iter().enumerate() {
    //     //     for right in buckets.iter().skip(i + 1) {
    //     //         assert!(
    //     //             !BlockSet { blocks: *left }
    //     //                 .any_blocks_merge_between(BlockSet { blocks: *right })
    //     //         );
    //     //     }
    //     // }
    //     BlockSet {
    //         blocks: buckets
    //             .into_iter()
    //             .map(|bucket| BlockSet { blocks: bucket
    // }.old_merge_blocks(ndim).blocks)             .fold(StackVec::new(), |acc,
    // bucket| acc.extend(bucket).unwrap()),     }
    // }
    // #[inline(never)]
    // fn merge_blocks_along_axis<const N: usize>(
    //     mut blocks: StackVec<Block, N>,
    //     _axis: usize,
    // ) -> StackVec<Block, N> {
    //     // TODO: use that we know the axis
    //     let ndim = 4;
    //     if blocks.len() <= 1 {
    //         return blocks;
    //     }
    //     loop {
    //         let mut merged_blocks = StackVec::new();
    //         'b1: for b1 in blocks {
    //             for b2 in &mut merged_blocks {
    //                 if let Some(merged) = b1.try_merge(*b2, ndim) {
    //                     *b2 = merged; // replace with merged block
    //                     continue 'b1;
    //                 }
    //             }
    //             merged_blocks = merged_blocks.push(b1).unwrap();
    //         }
    //         if blocks.len() == merged_blocks.len() {
    //             return blocks;
    //         }
    //         blocks = merged_blocks;
    //     }
    // }
    // #[must_use]
    // #[inline(never)]
    // fn merge_blocks(mut self, _ndim: usize) -> Self {
    //     let ndim = 4;
    //     let mut merged = [None; crate::MAX_BLOCKS];
    //     let mut unmerged = self.blocks;
    //     loop {
    //         'outer: for rhs in &mut unmerged {
    //             let any_merged = false;
    //             let slot = None;
    //             for lhs_option in &mut merged {
    //                 if let Some(lhs) = lhs_option {
    //                     if let Some(merged_block) = rhs.try_merge(*lhs, ndim) {
    //                         *rhs = merged_block;
    //                         *lhs_option = None;
    //                         any_merged = true;
    //                         continue 'outer;
    //                     }
    //                 } else {
    //                     slot = Some(lhs_option);
    //                 }
    //             }
    //             if !any_merged {
    //                 let slot = slot.unwrap_or(default);
    //                 *slot = rhs;
    //             }
    //         }
    //     }
    //     Self {
    //         blocks: StackVec::from_iter(merged.into_iter().flatten()).unwrap(),
    //     }
    // }
    // #[must_use]
    // #[inline(never)]
    // fn merge_blocks(mut self, _ndim: usize) -> Self {
    //     let ndim = 4;
    // self.blocks = self.blocks.sorted_unstable();
    //     loop {
    //         let mut new_blocks = StackVec::new();
    //         let mut i = 0;
    //         let mut any_merged = false;
    //         while i + 1 < self.blocks.len() {
    //             if let Some(merged) = self.blocks[i].try_merge(self.blocks[i +
    // 1], ndim) {                 any_merged = true;
    //                 new_blocks = new_blocks.push(merged).unwrap();
    //                 i += 2;
    //             } else {
    //                 new_blocks = new_blocks.push(self.blocks[i]).unwrap();
    //                 i += 1;
    //             }
    //         }
    //         self.blocks = new_blocks;
    //         if !any_merged {
    //             return BlockSet {
    //                 blocks: self.blocks,
    //             }.old_merge_blocks(ndim);
    //         }
    //     }
    // }
    // #[inline(never)]
    // fn insert_and_merge<const N: usize>(blocks: &mut StackVec<Block, N>, block:
    // Block) -> bool {     for b in blocks.iter_mut() {
    //         if let Some(merged) = block.try_merge(*b, 4) {
    //             *b = merged;
    //             return true;
    //         }
    //     }
    //     *blocks = blocks.push(block).unwrap();
    //     false
    // }
    // #[inline(never)]
    // fn insert_and_merge_along_axis<const N: usize>(
    //     blocks: &mut StackVec<Block, N>,
    //     block: Block,
    //     _axis: usize,
    // ) -> bool {
    //     Self::insert_and_merge(blocks, block)
    // }
    // #[inline(never)]
    // fn merge_blocks(mut self, _ndim: usize) -> Self {
    //     // TODO: there's probably some good theoretical max on the capacity
    // TODO: make this 7^3 instead of 8^3
    // TODO: make this 6^3 instead of 8^3 (0 and 5 are invalid along any axis)
    //     let mut buckets: [StackVec<Block, 3>; 512] = [StackVec::new(); 512];
    //     let mut bucket_is: StackVec<usize, { crate::MAX_BLOCKS }> =
    // StackVec::new();     // loop {
    //     // {
    //     for _loop_i in 0..2 {
    //         let mut any_merged = false;
    //         for axis in 0..4 {
    //             // TODO: if we merge when inserting, we can lower the capacity
    //             for bucket in &mut buckets {
    //                 // bucket.elems.fill(Block::invalid());
    //                 bucket.clear();
    //             }
    //             bucket_is.clear();

    //             #[inline(never)]
    //             fn populate_buckets<const N: usize>(
    //                 blocks: StackVec<Block, { crate::MAX_BLOCKS }>,
    //                 buckets: &mut [StackVec<Block, N>],
    //                 bucket_is: &mut StackVec<usize, { crate::MAX_BLOCKS }>,
    //                 axis: usize,
    //             ) -> bool {
    //                 // TODO: try_merge().unwrap() can be optimized
    //                 // TODO: try_merge_along_axis()
    //                 // TODO: actually merging can fail if the attitudes disagree
    //                 let mut any_merged = false;
    //                 for block in blocks {
    //                     #[inline(never)]
    //                     fn bucket_i_for_block(block: Block, axis: usize) -> usize
    // {                         let mut bucket_i = 0;
    //                         for shift in 0..4 {
    //                             if shift == axis {
    //                                 continue;
    //                             }
    //                             bucket_i <<= 3;
    //                             bucket_i |= block.layers().to_u16() >> (shift *
    // 4) & 0b111;                         } bucket_i as usize } let bucket_i =
    //    bucket_i_for_block(block, axis); *bucket_is =
    //    bucket_is.push(bucket_i).unwrap(); // buckets[bucket_i] =
    // buckets[bucket_i].push(block).unwrap();                     any_merged |=
    // BlockSet::insert_and_merge_along_axis(                         &mut
    // buckets[bucket_i],                         block,
    //                         axis,
    //                     );
    //                     // match buckets[bucket_i].len() {
    //                     //     0 => buckets[bucket_i] =
    // buckets[bucket_i].push(block).unwrap(),                     //     1 => {
    //                     //         if let Some(merged) =
    // buckets[bucket_i][0].try_merge(block, 4) {                     //
    // buckets[bucket_i][0] = merged;                     //
    // any_merged = true;                     //         } else {
    //                     //             buckets[bucket_i] =
    // buckets[bucket_i].push(block).unwrap();                     //         }
    //                     //     }
    //                     //     2 => {
    //                     //         if let Some(merged) =
    // buckets[bucket_i][0].try_merge(block, 4) {                     //
    // buckets[bucket_i][0] = merged;                     //         } else {
    //                     //             buckets[bucket_i][1] =
    //                     //                 buckets[bucket_i][1].try_merge(block,
    // 4).unwrap();                     //             if let Some(merged) =
    //                     //
    // buckets[bucket_i][0].try_merge(buckets[bucket_i][1], 4)
    // //             {                     //
    // buckets[bucket_i][0] = merged;                     //
    // buckets[bucket_i].pop();                     //             }
    //                     //         }
    //                     //         any_merged = true;
    //                     //     }
    //                     //     _ => unreachable!("too many blocks in bucket"),
    //                     // }
    //                 }
    //                 any_merged
    //             }
    //             any_merged |= populate_buckets(self.blocks, &mut buckets, &mut
    // bucket_is, axis);

    //             // #[inline(never)]
    //             // fn concatenate_blocks<const N: usize>(
    //             //     blocks: &mut StackVec<Block, { crate::MAX_BLOCKS }>,
    //             //     buckets: &mut [StackVec<Block, N>],
    //             //     bucket_is: StackVec<usize, { crate::MAX_BLOCKS }>,
    //             //     axis: usize,
    //             // ) -> bool {
    //             //     blocks.clear();
    //             //     let mut any_merged = false;
    //             //     for bucket_i in bucket_is.sorted_unstable().deduped() {
    //             //         let merged =
    // BlockSet::merge_blocks_along_axis(buckets[bucket_i], axis);
    // //         assert!(merged.len() <= buckets[bucket_i].len());
    // //         if merged.len() < buckets[bucket_i].len() {             //
    // any_merged = true;             //         }
    //             //         *blocks = blocks.extend(merged).unwrap();
    //             //     }
    //             //     any_merged
    //             // }
    //             #[inline(never)]
    //             fn concatenate_blocks<const N: usize>(
    //                 blocks: &mut StackVec<Block, { crate::MAX_BLOCKS }>,
    //                 buckets: &mut [StackVec<Block, N>],
    //                 bucket_is: StackVec<usize, { crate::MAX_BLOCKS }>,
    //                 _axis: usize,
    //             ) -> bool {
    //                 blocks.clear();
    //                 for bucket_i in bucket_is {
    //                     *blocks = blocks.extend(buckets[bucket_i]).unwrap();
    //                 }
    //                 false
    //             }
    //             dbg!(self.blocks);
    //             any_merged |= concatenate_blocks(&mut self.blocks, &mut buckets,
    // bucket_is, axis);         }
    //         if !any_merged {
    //             // LEN_AFTER[loop_i].fetch_add(1,
    // std::sync::atomic::Ordering::Relaxed);             return self;
    //         }
    //     }
    //     // after two passes, in almost all cases everything that can be merged
    // has been merged     self
    // }

    // #[inline(never)]
    // fn merge_blocks_along_axis(&mut self, axis: usize) -> bool {
    //     let mut any_merged = false;
    //     // let mask = match axis {
    //     //     0 => 0xFFF0,
    //     //     1 => 0xFF0F,
    //     //     2 => 0xF0FF,
    //     //     3 => 0x0FFF,
    //     //     _ => unreachable!(),
    //     // };
    //     let mask = !(0x7 << (axis * 4));
    //     self.blocks = self
    //         .blocks
    //         .sorted_unstable_by_key(|block| block.layers().to_u16() & mask);
    //     let chunks = self
    //         .blocks
    //         .iter()
    //         .chunk_by(|block| block.layers().to_u16() & mask);
    //     let mut new_blocks = StackVec::new();
    //     for (_key, chunk) in &chunks {
    //         let chunk: StackVec<Block, 3> =
    // StackVec::from_iter(chunk.cloned()).unwrap();         let merged =
    // Self::old_merge_stackvec_blocks_along_axis(chunk, axis);         assert!
    // (merged.len() <= chunk.len());         if merged.len() < chunk.len() {
    //             any_merged = true;
    //         }
    //         new_blocks = new_blocks.extend_from_stackvec(&merged).unwrap();
    //         // LEN_BEFORE[chunk.len()].fetch_add(1,
    // std::sync::atomic::Ordering::Relaxed);         //
    // LEN_AFTER[merged.len()].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    //     }
    //     self.blocks = new_blocks;
    //     any_merged
    // }
    // #[inline(never)]
    // fn merge_blocks(mut self, _ndim: usize) -> Self {
    //     // return self.old_merge_blocks(_ndim);
    //     // loop {
    //     for loop_i in 0.. {
    //         let mut any_merged = false;
    //         for axis in 0..4 {
    //             any_merged |= self.merge_blocks_along_axis(axis);
    //         }
    //         if !any_merged {
    //             // LEN_AFTER[loop_i].fetch_add(1,
    // std::sync::atomic::Ordering::Relaxed);             return self;
    //         }
    //     }
    //     // unreachable!()
    //     self
    // }
    #[must_use]
    #[inline(never)]
    fn merge_blocks(self, _ndim: usize) -> Self {
        // half the time len <= 8 and half the time 8 < len <= 16
        match self.blocks.len() {
            0 => unreachable!("empty BlockSet"),
            1 => self,
            2..=8 => self.merge_blocks_len_lte::<8>(_ndim),
            9..=16 => self.merge_blocks_len_lte::<16>(_ndim),
            17..=crate::MAX_BLOCKS => {
                // this happens basically none of the time
                self.old_merge_blocks(_ndim)
            }
            _ => unreachable!(),
        }
    }
    // #[must_use]
    // #[inline(never)]
    // // cargo asm --lib --rust robodoan::sim::blockbuilding::state::BlockSet::merge_blocks_len_lte_16 > temp.asm
    // // samply record cargo run --profile profiling
    // /// like we're seeing if we can maybe merge blocks[..] with blocks[offset..]
    // fn merge_blocks_len_lte_16(self, _ndim: usize) -> Self {
    //     const N: usize = 16;
    //     assert!(self.blocks.len() <= N);

    //     fn into_block((layer, attitude): (u16, ElemId)) -> Block {
    //         debug_assert_ne!(layer, 0);
    //         debug_assert_ne!(attitude, ElemId(u8::MAX));
    //         Block {
    //             layers: PackedLayers::from_u16(layer),
    //             attitude,
    //         }
    //     }
    //     let (mut layers, mut attitudes) = {
    //         let mut layers = [0; N];
    //         let mut attitudes = [ElemId(u8::MAX); N];
    //         for (i, block) in self.blocks.iter().enumerate() {
    //             layers[i] = block.layers().to_u16();
    //             attitudes[i] = block.attitude();
    //         }
    //         (Simd::from_array(layers), attitudes)
    //     };
    //     'outer: for loop_i in 0.. {
    //         let mut any_merged = false;
    //         // TODO: upper bound can be < 24, maybe initial self.blocks.len()?
    //         // TODO: this has to be strict maybe
    //         'offset: for offset in 1..self.blocks.len() {
    //             // let offset_layers = {
    //             //     let mut offset_layers = [0; N];
    //             //     for (i, layer) in layers.iter().enumerate() {
    //             //         offset_layers[(offset + i) % N] = *layer;
    //             //     }
    //             //     offset_layers
    //             // };
    //             #[inline(always)]
    //             // fn get_offset_layers(layers: &[u16; N], offset: usize) -> [u16; N] {
    //             fn get_offset_layers(layers: Simd<u16, N>, offset: usize) -> Simd<u16, N> {
    //                 // let mut offset_layers = Simd::default();
    //                 // offset_layers[..N - offset].copy_from_slice(&layers[offset..]);
    //                 // offset_layers
    //                 // layers.shift_elements_left::<{ offset }>(0)

    //                 // lol
    //                 match offset {
    //                     1 => layers.shift_elements_left::<1>(0),
    //                     2 => layers.shift_elements_left::<2>(0),
    //                     3 => layers.shift_elements_left::<3>(0),
    //                     4 => layers.shift_elements_left::<4>(0),
    //                     5 => layers.shift_elements_left::<5>(0),
    //                     6 => layers.shift_elements_left::<6>(0),
    //                     7 => layers.shift_elements_left::<7>(0),
    //                     8 => layers.shift_elements_left::<8>(0),
    //                     9 => layers.shift_elements_left::<9>(0),
    //                     10 => layers.shift_elements_left::<10>(0),
    //                     11 => layers.shift_elements_left::<11>(0),
    //                     12 => layers.shift_elements_left::<12>(0),
    //                     13 => layers.shift_elements_left::<13>(0),
    //                     14 => layers.shift_elements_left::<14>(0),
    //                     15 => layers.shift_elements_left::<15>(0),
    //                     _ => unreachable!(),
    //                 }
    //             }
    //             let offset_layers = get_offset_layers(layers, offset);

    //             #[inline(always)]
    //             // fn can_maybe_merge(
    //             //     layers: &[u16; N],
    //             //     offset_layers: &[u16; N],
    //             //     offset: usize,
    //             // ) -> [bool; N] {
    //             fn can_maybe_merge(layers: Simd<u16, N>, offset_layers: Simd<u16, N>) -> u16 {
    //                 // let mut can_maybe_merge = [false; N];
    //                 // // for i in 0..N {
    //                 // // for i in offset..N {
    //                 // for i in 0..N {
    //                 //     let layer_difference = layers[i] ^ offset_layers[i];
    //                 //     if layers[i] != 0 && offset_layers[i] != 0 {
    //                 //         debug_assert!(layers[i] != offset_layers[i]);
    //                 //         debug_assert!(layer_difference.trailing_zeros() as usize / 4 < 4);
    //                 //     }
    //                 //     can_maybe_merge[i] = (layers[i] != 0)
    //                 //         && (offset_layers[i] != 0)
    //                 //         && ((layer_difference
    //                 //             & !(0x7
    //                 //                 << ((layer_difference.trailing_zeros() as usize / 4) * 4)))
    //                 //             == 0);
    //                 // }
    //                 // can_maybe_merge
    //                 let layer_difference = layers ^ offset_layers;

    //                 ((layers.simd_ne(Simd::splat(0)))
    //                     & (offset_layers.simd_ne(Simd::splat(0)))
    //                     & (layer_difference
    //                         & !(Simd::splat(0x7)
    //                             << (layer_difference.trailing_zeros() / Simd::splat(4)
    //                                 * Simd::splat(4))))
    //                     .simd_eq(Simd::splat(0)))
    //                 .to_bitmask() as u16
    //             }
    //             let can_maybe_merge = can_maybe_merge(layers, offset_layers);
    //             // if !can_maybe_merge.iter().any(|&b| b) {
    //             if can_maybe_merge == 0 {
    //                 continue 'offset;
    //             }
    //             // for i in 0..blocks.len() {
    //             // for i in offset..N {
    //             for i in 0..N - offset {
    //                 // if !can_maybe_merge[i] {
    //                 if (can_maybe_merge & (1 << i)) == 0 {
    //                     continue;
    //                 }
    //                 // debug_assert!(i >= offset);
    //                 let offset_i = i + offset;
    //                 debug_assert_eq!(offset_layers[i], layers[offset_i]);
    //                 // debug_assert_eq!(blocks[i].0, layers[i]);
    //                 // debug_assert_eq!(blocks[offset_i].0, offset_layers[i]);
    //                 // {
    //                 //     let lhs = blocks[i].0;
    //                 //     let rhs = blocks[offset_i].0;
    //                 //     let layer_difference = lhs ^ rhs;
    //                 //     debug_assert!(
    //                 //         (lhs != 0)
    //                 //             && (rhs != 0)
    //                 //             && ((layer_difference
    //                 //                 & !(0x7
    //                 //                     << ((layer_difference.trailing_zeros() as usize / 4)
    //                 //                         * 4)))
    //                 //                 == 0)
    //                 //     );
    //                 // }
    //                 // blocks[(i + offset) % blocks.len()
    //                 if let Some(merged) = into_block((layers[i], attitudes[i]))
    //                     .try_merge(into_block((layers[offset_i], attitudes[offset_i])), 4)
    //                 {
    //                     // TODO: we can either reset the offset loop now
    //                     // or finish the offset loop
    //                     any_merged = true;
    //                     // blocks[i] = (merged.layers().to_u16(), merged.attitude());
    //                     // blocks[offset_i] = (0, ElemId::IDENT);
    //                     // blocks[(i + offset) % blocks.len()] = (0, ElemId::IDENT);
    //                     // TODO: layers has a small delta, do don't recompute all of it
    //                     // layers = get_layers(&blocks);
    //                     layers[i] = merged.layers().to_u16();
    //                     layers[offset_i] = 0;
    //                     attitudes[i] = merged.attitude();
    //                     attitudes[offset_i] = ElemId(u8::MAX);
    //                     // we just merged blocks[i] and blocks[(i + offset) % len], but it'll also
    //                     // want to merge blocks[(i + offset) % len] with blocks[i] later
    //                     // TODO: really we should restart at the same offset
    //                     // and in that case, offset_layers will have a small delta
    //                     // continue 'offset;
    //                     continue 'outer;
    //                 }
    //             }
    //         }
    //         if !any_merged {
    //             // LEN_AFTER[loop_i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    //             return BlockSet {
    //                 blocks: StackVec::<Block, { crate::MAX_BLOCKS }>::from_iter(
    //                     layers[..self.blocks.len()]
    //                         .iter()
    //                         .zip(attitudes[..self.blocks.len()].iter())
    //                         .filter_map(|(&layer, &attitude)| {
    //                             if layer == 0 {
    //                                 None
    //                             } else {
    //                                 Some(into_block((layer, attitude)))
    //                             }
    //                         }),
    //                 )
    //                 .unwrap(),
    //             };
    //         }
    //     }
    //     unreachable!()
    // }
    // #[must_use]
    // #[inline(never)]
    // // cargo asm --lib --rust robodoan::sim::blockbuilding::state::BlockSet::merge_blocks_len_lte_16 > temp.asm
    // // samply record cargo run --profile profiling
    // fn merge_blocks_len_lte_16(self, _ndim: usize) -> Self {
    //     const N: usize = 16;
    //     assert!(!self.blocks.is_empty());
    //     assert!(self.blocks.len() <= N);

    //     fn into_block((layer, attitude): (u16, ElemId)) -> Block {
    //         debug_assert_ne!(layer, 0);
    //         debug_assert_ne!(attitude, ElemId(u8::MAX));
    //         Block {
    //             layers: PackedLayers::from_u16(layer),
    //             attitude,
    //         }
    //     }
    //     let (mut layers, mut attitudes) = {
    //         let mut layers = [0; N];
    //         let mut attitudes = [ElemId(u8::MAX); N];
    //         for (i, block) in self.blocks.iter().enumerate() {
    //             layers[i] = block.layers().to_u16();
    //             attitudes[i] = block.attitude();
    //         }
    //         (Simd::from_array(layers), attitudes)
    //     };
    //     'outer: for loop_i in 0.. {
    //         let mut any_merged = false;
    //         //   0 1 2 3 4 5 6 7 8
    //         // 0   * * * * * * * *
    //         // 1     * * * * * * *
    //         // 2       * * * * * *
    //         // 3         * * * * *
    //         // 4           * * * *
    //         // 5             * * *
    //         // 6               * *
    //         // 7                 *
    //         // ->
    //         //   0 1 2 3 4 5 6 7 8
    //         // 0 . * * * * * * * *
    //         // 1 . . * * * * * * *
    //         // 2 . . . * * * * * *
    //         // 3 . . . . * * * * *
    //         // 4           . . . .
    //         // 5             . . .
    //         // 6               . .
    //         // 7                 .
    //         // let mut layer_i = 0;
    //         // 'layer: while layer_i + 1 < self.blocks.len() {
    //         // 'layer: for layer_i in 0..(self.blocks.len() - 1) {
    //         // 'layer: for layer_i in 0..N / 2 {
    //         'layer: for layer_i in 0..N - 1 {
    //             let layer = layers[layer_i];
    //             if layer == 0 {
    //                 // layer_i += 1;
    //                 continue 'layer;
    //             }
    //             // TODO: multiple layer spats at once
    //             // // layer_splat indices:   [ 3 3 3 3 3 3 3 3 ]
    //             // // masked_layers indices: [ _ _ _ _ 4 5 6 7 ]
    //             // let layer_splat = Simd::splat(layer);
    //             // layer_splat indices:   [ 7 7 7 7 3 3 3 3 ]
    //             // masked_layers indices: [ 0 1 2 3 4 5 6 7 ]
    //             let layer_splat = Simd::splat(layer);
    //             let masked_layers = get_masked_layers(layers, layer_i);
    //             #[inline(always)]
    //             fn get_masked_layers(layers: Simd<u16, N>, layer_i: usize) -> Simd<u16, N> {
    //                 let mut ret = layers;
    //                 ret[..=layer_i].fill(0);
    //                 ret
    //             }

    //             let can_maybe_merge = can_maybe_merge(layer_splat, masked_layers);
    //             #[inline(always)]
    //             fn can_maybe_merge(lhs: Simd<u16, N>, rhs: Simd<u16, N>) -> u16 {
    //                 const ZERO: Simd<u16, N> = Simd::from_array([0; N]);
    //                 const SEVEN: Simd<u16, N> = Simd::from_array([7; N]);
    //                 // const FOUR: Simd<u16, N> = Simd::from_array([4; N]);
    //                 const NOT_THREE: Simd<u16, N> = Simd::from_array([!3; N]);

    //                 let layer_difference = lhs ^ rhs;
    //                 ((lhs.simd_ne(ZERO))
    //                     & (rhs.simd_ne(ZERO))
    //                     & (layer_difference
    //                         // & !(SEVEN << (layer_difference.trailing_zeros() / FOUR * FOUR)))
    //                         & !(SEVEN << (layer_difference.trailing_zeros() & NOT_THREE)))
    //                         .simd_eq(ZERO))
    //                 .to_bitmask() as u16
    //             }

    //             // if !can_maybe_merge.iter().any(|&b| b) {
    //             if can_maybe_merge == 0 {
    //                 continue 'layer;
    //             }
    //             // debug_assert_eq!(layers[layer_i + 1], masked_layers[0]);
    //             debug_assert_eq!(masked_layers[layer_i], 0);
    //             debug_assert_eq!(masked_layers[layer_i + 1], layers[layer_i + 1]);
    //             // for j in 0..N - (layer_i + 1) {
    //             for j in 0..N {
    //                 if (can_maybe_merge & (1 << j)) == 0 {
    //                     continue;
    //                 }
    //                 // let offset_i = layer_i + 1 + j;
    //                 // let offset_i = (layer_i + 1 + j) % N;
    //                 let other_i = j;
    //                 debug_assert_eq!(masked_layers[j], layers[other_i]);
    //                 // debug_assert!(layer_i < offset_i);
    //                 // debug_assert!(i >= offset);
    //                 // let offset_i = i + offset;

    //                 // debug_assert_eq!(offset_layers[i], layers[offset_i]);
    //                 if let Some(merged) = into_block((layers[layer_i], attitudes[layer_i]))
    //                     .try_merge(into_block((layers[other_i], attitudes[other_i])), 4)
    //                 {
    //                     // TODO: we can either reset the offset loop now
    //                     // or finish the offset loop
    //                     any_merged = true;
    //                     // blocks[i] = (merged.layers().to_u16(), merged.attitude());
    //                     // blocks[offset_i] = (0, ElemId::IDENT);
    //                     // blocks[(i + offset) % blocks.len()] = (0, ElemId::IDENT);
    //                     layers[layer_i] = merged.layers().to_u16();
    //                     layers[other_i] = 0;
    //                     attitudes[layer_i] = merged.attitude();
    //                     attitudes[other_i] = ElemId(u8::MAX);
    //                     // we just merged blocks[i] and blocks[(i + offset) % len], but it'll also
    //                     // want to merge blocks[(i + offset) % len] with blocks[i] later
    //                     // TODO: really we should restart at the same offset
    //                     // and in that case, offset_layers will have a small delta
    //                     continue 'outer;
    //                     // // don't increase layer_i
    //                     // continue 'layer;
    //                     // TODO: try not immediately continuing
    //                 }
    //             }
    //             // if any_merged {
    //             //     continue 'outer;
    //             // }
    //             // layer_i += 1;
    //         }
    //         if !any_merged {
    //             // LEN_AFTER[loop_i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    //             return BlockSet {
    //                 blocks: StackVec::<Block, { crate::MAX_BLOCKS }>::from_iter(
    //                     layers[..self.blocks.len()]
    //                         .iter()
    //                         .zip(attitudes[..self.blocks.len()].iter())
    //                         .filter_map(|(&layer, &attitude)| {
    //                             if layer == 0 {
    //                                 None
    //                             } else {
    //                                 Some(into_block((layer, attitude)))
    //                             }
    //                         }),
    //                 )
    //                 .unwrap(),
    //             };
    //         }
    //     }
    //     unreachable!()
    // }
    #[must_use]
    #[inline(never)]
    // cargo asm --lib --rust robodoan::sim::blockbuilding::state::BlockSet::merge_blocks_len_lte_16 > temp.asm
    // samply record cargo run --profile profiling
    /// try to merge blocks with blocks.rotated(offset_i)
    fn can_maybe_merge_blocks_len_lte_16(self) -> bool {
        // TODO: fast can_merge
        const N: usize = 16;
        assert!(!self.blocks.is_empty());
        assert!(self.blocks.len() <= N);

        fn into_block((layer, attitude): (u16, ElemId)) -> Block {
            debug_assert_ne!(layer, 0);
            debug_assert_ne!(attitude, ElemId(u8::MAX));
            Block {
                layers: PackedLayers::from_u16(layer),
                attitude,
            }
        }
        let layers = {
            let mut layers = [0; N];
            // TODO: attitudes is unused
            let mut attitudes = [ElemId(u8::MAX); N];
            for (i, block) in self.blocks.iter().enumerate() {
                layers[i] = block.layers().to_u16();
                attitudes[i] = block.attitude();
            }
            Simd::from_array(layers)
        };
        // let mut ret = Mask::from_bitmask(0);
        const STRIDE: usize = 4;
        for offset_i_unstrode in 0..N / 2 / STRIDE {
            let mut stride_mask = [Mask::from_bitmask(0); STRIDE];
            for (stride_i, mask) in stride_mask.iter_mut().enumerate() {
                let offset_i = offset_i_unstrode * STRIDE + stride_i + 1;
                // layers indices:        [ 0 1 2 3 ] [ 0 1 2 3 ]
                // offset_layers indices: [ 1 2 3 0 ] [ 2 3 0 1 ]
                let offset_layers = get_offset_layers(layers, offset_i);
                #[inline(always)]
                fn get_offset_layers(layers: Simd<u16, N>, offset_i: usize) -> Simd<u16, N> {
                    match offset_i {
                        1 => layers.rotate_elements_left::<1>(),
                        2 => layers.rotate_elements_left::<2>(),
                        3 => layers.rotate_elements_left::<3>(),
                        4 => layers.rotate_elements_left::<4>(),
                        5 => layers.rotate_elements_left::<5>(),
                        6 => layers.rotate_elements_left::<6>(),
                        7 => layers.rotate_elements_left::<7>(),
                        8 => layers.rotate_elements_left::<8>(),
                        _ => unreachable!(),
                    }
                }

                *mask = {
                    const ZERO: Simd<u16, N> = Simd::from_array([0; N]);
                    const SEVEN: Simd<u16, N> = Simd::from_array([7; N]);
                    // const FOUR: Simd<u16, N> = Simd::from_array([4; N]);
                    const NOT_THREE: Simd<u16, N> = Simd::from_array([!3; N]);

                    let layer_difference = layers ^ offset_layers;
                    layers.simd_ne(ZERO)
                        & offset_layers.simd_ne(ZERO)
                        & (layer_difference
                            & !(SEVEN << (layer_difference.trailing_zeros() & NOT_THREE)))
                            .simd_eq(ZERO)
                };
            }
            for mask in &stride_mask {
                if mask.any() {
                    return true;
                }
            }
        }
        false
    }
    #[must_use]
    #[inline(never)]
    // cargo asm --lib --rust robodoan::sim::blockbuilding::state::BlockSet::merge_blocks_len_lte 0 > temp.asm
    // samply record cargo run --profile profiling
    /// try to merge blocks with blocks.rotated(offset_i)
    fn merge_blocks_len_lte<const N: usize>(self, _ndim: usize) -> Self
    where
        LaneCount<N>: SupportedLaneCount,
    {
        // const N: usize = 16;
        // assert!(N == 8 || N == 16);
        // assert!(!self.blocks.is_empty());
        // assert!(self.blocks.len() <= N);
        match N {
            8 => assert!((2..=8).contains(&self.blocks.len())),
            16 => assert!((9..=16).contains(&self.blocks.len())),
            _ => panic!("bad N"),
        }

        let simd_zero: Simd<u16, N> = Simd::from_array([0; N]);
        let seven: Simd<u16, N> = Simd::from_array([7; N]);
        // const FOUR: Simd<u16, N> = Simd::from_array([4; N]);
        let not_three: Simd<u16, N> = Simd::from_array([!3; N]);

        fn into_block((layer, attitude): (u16, ElemId)) -> Block {
            debug_assert_ne!(layer, 0);
            debug_assert_ne!(attitude, ElemId(u8::MAX));
            Block {
                layers: PackedLayers::from_u16(layer),
                attitude,
            }
        }
        let (mut layers, mut attitudes) = {
            let mut layers = [0; N];
            let mut attitudes = [ElemId(u8::MAX); N];
            for (i, block) in self.blocks.iter().enumerate() {
                layers[i] = block.layers().to_u16();
                attitudes[i] = block.attitude();
            }
            (Simd::from_array(layers), attitudes)
        };
        let mut layers_nonzero = layers.simd_ne(simd_zero);
        let mut ever_maybe_merged = false;
        let mut ever_merged = false;
        let mut merged_count = 0;
        let mut maybe_merged_count = 0;
        'outer: for loop_i in 0.. {
            let mut any_merged = false;
            'layer: for offset_i in 1..=N / 2 {
                // TODO: stride
                // let layer_splat = Simd::splat(layer);
                // for N = 4
                // layers indices:        [ 0 1 2 3 ] [ 0 1 2 3 ]
                // offset_layers indices: [ 1 2 3 0 ] [ 2 3 0 1 ]
                // for N = 8
                // layers  : [ 0 1 2 3 4 5 6 7 ]
                // offset 1: [ 1 2 3 4 5 6 7 0 ]
                // offset 2: [ 2 3 4 5 6 7 0 1 ]
                // offset 3: [ 3 4 5 6 7 0 1 2 ]
                // offset 4: [ 4 5 6 7 0 1 2 3 ]
                // for N = 8, blocks.len() = 6
                // layers  : [ 0 1 2 3 4 5 _ _ ]
                // offset 1: [ 1 2 3 4 5 _ _ 0 ]
                // offset 2: [ 2 3 4 5 _ _ 0 1 ]
                // offset 3: [ 3 4 5 _ _ 0 1 2 ]
                // offset 4: [ 4 5 _ _ 0 1 2 3 ]
                // for N = 8, blocks.len() = 4
                // layers  : [ 0 1 2 3 _ _ _ _ ]
                // offset 1: [ 1 2 3 _ _ _ _ 0 ]
                // offset 2: [ 2 3 _ _ _ _ 0 1 ]
                // offset 3: [ 3 _ _ _ _ 0 1 2 ]
                // offset 4: [ _ _ _ _ 0 1 2 3 ]
                // TODO: we can pack them tighter and then do blocks.len() / 2 iterations

                let offset_layers = {
                    match offset_i {
                        1 => layers.rotate_elements_left::<1>(),
                        2 => layers.rotate_elements_left::<2>(),
                        3 => layers.rotate_elements_left::<3>(),
                        4 => layers.rotate_elements_left::<4>(),
                        5 => layers.rotate_elements_left::<5>(),
                        6 => layers.rotate_elements_left::<6>(),
                        7 => layers.rotate_elements_left::<7>(),
                        8 => layers.rotate_elements_left::<8>(),
                        _ => unreachable!(),
                    }
                };

                let can_maybe_merge = {
                    debug_assert_eq!(layers.simd_ne(simd_zero), layers_nonzero);
                    let layer_difference = layers ^ offset_layers;
                    let nonzero = layers_nonzero & offset_layers.simd_ne(simd_zero);
                    // // this is rare
                    // if !nonzero.any() {
                    //     continue;
                    // }
                    (nonzero
                        & (layer_difference
                            // & !(SEVEN << (layer_difference.trailing_zeros() / FOUR * FOUR)))
                            & !(seven << (layer_difference.trailing_zeros() & not_three)))
                            .simd_eq(simd_zero))
                    .to_bitmask()
                };

                if can_maybe_merge == 0 {
                    continue 'layer;
                }
                ever_maybe_merged = true;
                maybe_merged_count += 1;
                debug_assert_eq!(offset_layers[0], layers[offset_i]);
                // while let Some(asdf) = can_maybe_merge.first_set() {
                for layer_i in 0..N {
                    if layer_i >= N {
                        break;
                    }
                    if (can_maybe_merge & (1 << layer_i)) == 0 {
                        continue;
                    }
                    let other_i = (offset_i + layer_i) % N;
                    debug_assert_eq!(offset_layers[layer_i], layers[other_i]);
                    // TODO: we can cache indistinguishable_attitudes
                    if let Some(merged) = into_block((layers[layer_i], attitudes[layer_i]))
                        .try_merge(into_block((layers[other_i], attitudes[other_i])), 4)
                    {
                        // TODO: we can either reset the offset loop now
                        // or finish the offset loop
                        ever_merged = true;
                        any_merged = true;
                        merged_count += 1;

                        layers[layer_i] = merged.layers().to_u16();
                        attitudes[layer_i] = merged.attitude();

                        layers[other_i] = 0;
                        attitudes[other_i] = ElemId(u8::MAX); // this is technically unnecessary
                        layers_nonzero.set(other_i, false);

                        // we just merged blocks[i] and blocks[(i + offset) % len], but it'll also
                        // want to merge blocks[(i + offset) % len] with blocks[i] later
                        // TODO: really we should restart at the same offset
                        // and in that case, offset_layers will have a small delta
                        // continue 'outer is probably faster because it's extra rare for two blocks to merge
                        continue 'outer;
                        // // don't increase layer_i
                        // continue 'layer;
                        // TODO: try not immediately continuing
                    }
                }
                // if any_merged {
                //     continue 'outer;
                // }
                // layer_i += 1;
            }
            if !any_merged {
                // LEN_AFTER[loop_i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug_assert!(maybe_merged_count >= merged_count);
                debug_assert_eq!(maybe_merged_count > 0, ever_maybe_merged);
                debug_assert_eq!(merged_count > 0, ever_merged);
                // if ever_merged {
                //     // ~2% of the time
                //     // total: 6863633
                //     // max: 3212191
                //     //   00: 000000000 0.00000
                //     //   01: 003212191 0.46800 ################################
                //     //   02: 001744312 0.25414 #################
                //     //   03: 001023720 0.14915 ##########
                //     //   04: 000431632 0.06289 ####
                //     //   05: 000226790 0.03304 ##
                //     //   06: 000096907 0.01412
                //     LEN_AFTER[maybe_merged_count]
                //         .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // } else {
                //     // ~98% of the time
                //     // total: 336331202
                //     // max: 167401276
                //     //   00: 167401276 0.49773 ################################
                //     //   01: 112773096 0.33530 #####################
                //     //   02: 040422711 0.12019 #######
                //     //   03: 011699681 0.03479 ##
                //     //   04: 003009690 0.00895
                //     LEN_BEFORE[maybe_merged_count]
                //         .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // }
                //
                // if we thought that we could maybe merge, we only ever merge ~5% of the time
                // heuristic / actual  actual = false  actual = true
                // heuristic = false   0.48947         0.00000
                // heuristic = true    0.48835         0.02218
                // LEN_AFTER[2 * (ever_maybe_merged as usize) + (ever_merged as usize)]
                //     .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // we only ever merge ~3% of the time, so this skips the iter stuff
                // assert!(ever_maybe_merge);
                if !ever_merged {
                    return self;
                } else {
                    return BlockSet {
                        blocks: StackVec::<Block, { crate::MAX_BLOCKS }>::from_iter(
                            layers[..self.blocks.len()]
                                .iter()
                                .zip(attitudes[..self.blocks.len()].iter())
                                .filter_map(|(&layer, &attitude)| {
                                    debug_assert_eq!(layer == 0, attitude == ElemId(u8::MAX));
                                    if layer == 0 {
                                        None
                                    } else {
                                        Some(into_block((layer, attitude)))
                                    }
                                }),
                        )
                        .unwrap(),
                    };
                }
            }
        }
        unreachable!()
    }

    /// Constructs a state containing the given blocks and combines blocks as
    /// much as possible.
    ///
    /// TODO: it may be possible for this to get an "N-perm situation" which
    /// would be bad.
    #[must_use]
    fn from_blocks(blocks: StackVec<Block, { crate::MAX_BLOCKS }>, ndim: usize) -> Self {
        Self { blocks }.merge_blocks(ndim)
    }

    pub fn is_solved(self) -> bool {
        self.blocks.len() == 1
    }

    fn fully_split_along(blocks: &[Block], grip: GripId) -> impl Iterator<Item = Block> {
        blocks
            .iter()
            .flat_map(move |block| block.split(grip))
            .flatten()
    }
    fn fully_split(self) -> HashSet<Block> {
        let mut blocks: Vec<Block> = self.blocks.to_vec();
        for grip in HYPERCUBE_GRIPS {
            blocks = Self::fully_split_along(&blocks, grip).collect();
        }
        let ret: HashSet<Block> = blocks.into_iter().collect();
        for (l, r) in ret.iter().sorted().tuple_windows() {
            assert_ne!(
                l.layers(),
                r.layers(),
                "same layers with different attitudes in {self}"
            );
        }
        ret
    }
    fn piece_equivalent(self, other: Self) -> bool {
        self.fully_split()
            .iter()
            .sorted()
            .zip(other.fully_split().iter().sorted())
            .all(|(lhs, rhs)| {
                lhs.layers() == rhs.layers()
                    && lhs.indistinguishable_attitudes(4).collect::<HashSet<_>>()
                        == rhs.indistinguishable_attitudes(4).collect::<HashSet<_>>()
            })
    }
    fn assert_piece_equivalent(self, other: Self) {
        let lhs = self.fully_split().into_iter().sorted().collect::<Vec<_>>();
        let rhs = other.fully_split().into_iter().sorted().collect::<Vec<_>>();
        for (l, r) in std::iter::zip(lhs, rhs) {
            // assert_eq!(l.layers(), r.layers());
            // assert_eq!(
            //     l.indistinguishable_attitudes(4).collect::<HashSet<_>>(),
            //     r.indistinguishable_attitudes(4).collect::<HashSet<_>>()
            // );
            assert_eq!(
                (
                    l.layers(),
                    l.indistinguishable_attitudes(4).collect::<HashSet<_>>()
                ),
                (
                    r.layers(),
                    r.indistinguishable_attitudes(4).collect::<HashSet<_>>()
                )
            );
        }
    }
    // fn assert_disjoint_blocks(self) {
    // this is wrong btw
    //     for (i, b1) in self.blocks.iter().enumerate() {
    //         for b2 in self.blocks.iter().skip(i + 1) {
    //             assert!(
    //                !b1.layers().is_subset_of(b2.layers()),
    //                 "overlapping blocks in {self}"
    //             );
    //             assert!(
    //                !b2.layers().is_subset_of(b1.layers()),
    //                 "overlapping blocks in {self}"
    //             );
    //         }
    //     }
    // }

    /// Applies `setup_moves` to each piece in `block` and then adds all the
    /// pieces into the puzzle state, except for the ones that are already in
    /// the puzzle state.
    ///
    /// `block` must have the identity attitude.
    ///
    /// Returns `None` if the puzzle state would have more than
    /// [`crate::MAX_BLOCKS`] blocks.
    #[must_use]
    pub fn add_block_with_setup_moves(
        self,
        puzzle: &Puzzle,
        setup_moves: &[Twist],
        new_block: Block,
    ) -> Option<Self> {
        let pieces_from_block = |block: Block| {
            assert_eq!(block.attitude(), crate::IDENT);
            let mut blocks = vec![block];
            for g in (puzzle.grip_set() & block.blocked_grips()).iter() {
                blocks = blocks
                    .into_iter()
                    .flat_map(|b| b.split(g))
                    .flatten() // Option<T> -> T
                    .collect();
            }
            blocks
                .into_iter()
                .map(|b| Piece::new_solved(b.active_grips().iter()))
        };

        let mut new_pieces = pieces_from_block(new_block).collect::<HashSet<Piece>>();
        for old_block in self.blocks {
            for piece in pieces_from_block(old_block.at_solved()) {
                new_pieces.remove(&piece);
            }
        }

        let init_piece = |new_piece| setup_moves.iter().fold(new_piece, |p, &twist| twist * p);

        Some(Self::from_blocks(
            self.blocks
                .extend(new_pieces.into_iter().map(init_piece).map(Block::from))?,
            puzzle.ndim,
        ))
    }
}
impl fmt::Display for BlockSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        write!(f, "{}", self.blocks.iter().join(", "))?;
        write!(f, "]")?;
        Ok(())
    }
}

pub fn hash_of_layer(layer: u16) -> u16 {
    let mut ret = 0;
    for axis in 0..4 {
        let bits = (layer >> (axis * 4)) & 0x7;
        ret *= 6;
        ret += match bits {
            0 => panic!("invalid"),
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => panic!("invalid"),
            6 => 4,
            7 => 5,
            _ => unreachable!(),
        };
    }
    ret
}
pub fn layer_of_hash(mut hash: u16) -> PackedLayers {
    let mut ret = 0;
    for axis in 0..4 {
        let bits = hash % 6;
        hash /= 6;
        let layer_bits = match bits {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 6,
            5 => 7,
            _ => panic!("invalid"),
        };
        // TODO: this might not be correct
        ret |= layer_bits << (axis * 4);
    }
    PackedLayers::from_u16(ret)
}

#[cfg(test)]
mod tests {
    use crate::StackVec;
    use crate::sim::*;

    #[test]
    fn test_puzzle_state() {
        let last_layer_algs = [
            (true, ""),
            (false, "R"),
            (false, "D"),
            (true, "U"),
            (true, "R R'"),
            (true, "D D'"),
            (true, "R L R' L'"),
            (true, "F R U R' U' F'"),                       // fruruf
            (true, "R U R' U' R' F R F'"),                  // sexy sledgehammer
            (true, "R U R' F' R U R' U' R' F R2 U' R'"),    // J perm
            (true, "R U R' U R' U' R2 U' R' U R' U R"),     // U perm
            (true, "R U R' U' R' F R2 U' R' U' R U R' F'"), // T perm
        ];

        let ndim = 3;

        for (should_be_solved, last_layer_twist_seq) in last_layer_algs {
            let mut state = BlockSet {
                blocks: StackVec::from_iter([Block::new_solved([], [GripId::U]).unwrap()]).unwrap(),
            };
            for t in last_layer_twist_seq
                .split_ascii_whitespace()
                .map(|s| TWISTS_FROM_NAME[s])
            {
                state = state.do_twist(t, ndim).unwrap();
            }
            assert_eq!(should_be_solved, state.is_solved());
        }
    }
}
