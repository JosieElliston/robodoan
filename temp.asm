.section __TEXT,__text,regular,pure_instructions
	.p2align	2
robodoan::sim::blockbuilding::state::BlockSet::merge_blocks_len_lte:
Lfunc_begin324:
		// src/sim/blockbuilding/state.rs:1105
		fn merge_blocks_len_lte<const N: usize>(self, _ndim: usize) -> Self
	.cfi_startproc
	stp x28, x27, [sp, #-96]!
	.cfi_def_cfa_offset 96
	stp x26, x25, [sp, #16]
	stp x24, x23, [sp, #32]
	stp x22, x21, [sp, #48]
	stp x20, x19, [sp, #64]
	stp x29, x30, [sp, #80]
	add x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_remember_state
	sub x9, sp, #608
	and sp, x9, #0xffffffffffffffe0
		// src/stackvec.rs:182
		&self.elems[0..self.len as usize]
	ldrb w20, [x1]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:796
		if self < rhs {
	cmp w20, #22
	b.hs LBB324_71
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/range.rs:847
		(match self.start_bound() {
	sub w8, w20, #9
	cmp w8, #8
	b.hs LBB324_73
	mov x21, x1
	mov x19, x0
	add x22, sp, #480
Lloh1502:
	adrp x1, l_.memset_pattern.1@PAGE
Lloh1503:
	add x1, x1, l_.memset_pattern.1@PAGEOFF
	add x0, sp, #544
	mov w2, #32
	bl _memset_pattern16
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:547
		unsafe { crate::intrinsics::copy_nonoverlapping(src, dst, count) }
	ldp q0, q1, [x22, #64]
	stp q0, q1, [sp, #288]
Lloh1504:
	adrp x1, l_.memset_pattern@PAGE
Lloh1505:
	add x1, x1, l_.memset_pattern@PAGEOFF
	add x0, sp, #544
	mov w2, #32
	bl _memset_pattern16
	ldp q1, q0, [x22, #64]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:961
		unsafe { intrinsics::offset(self, count) }
	add x8, x20, x20, lsl #1
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1687
		self.as_ptr() == other.as_ptr()
	add x5, x8, #1
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ldur h5, [x21, #1]
	movi.2d v4, #0000000000000000
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w8, [x21, #3]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #4
	str x19, [sp, #8]
	b.ne LBB324_4
	mov w9, #255
	b LBB324_5
LBB324_4:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x9, x21, #4
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v5 }[1], [x9]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w9, [x21, #6]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #7
	b.ne LBB324_57
LBB324_5:
	mov w10, #255
LBB324_6:
	mov w11, #255
LBB324_7:
	mov w12, #255
LBB324_8:
	mov w13, #255
LBB324_9:
	mov w14, #255
LBB324_10:
	mov w15, #255
LBB324_11:
	mov w16, #255
LBB324_12:
	mov w17, #255
LBB324_13:
	mov w0, #255
LBB324_14:
	mov w1, #255
LBB324_15:
	mov w2, #255
LBB324_16:
	mov w3, #255
LBB324_17:
	mov w4, #255
LBB324_18:
	mov w5, #255
LBB324_19:
	mov w24, #0
		// src/sim/blockbuilding/state.rs:1132
		let (mut layers, mut attitudes) = {
	strb w8, [sp, #432]
	stp q5, q4, [sp, #384]
	strb w9, [sp, #433]
	strb w10, [sp, #434]
	strb w11, [sp, #435]
	strb w12, [sp, #436]
	strb w13, [sp, #437]
	strb w14, [sp, #438]
	strb w15, [sp, #439]
	strb w16, [sp, #440]
	strb w17, [sp, #441]
	strb w0, [sp, #442]
	strb w1, [sp, #443]
	strb w2, [sp, #444]
	strb w3, [sp, #445]
	strb w4, [sp, #446]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/simd/cmp/eq.rs:40
		unsafe { Mask::from_int_unchecked(core::intrinsics::simd::simd_ne(self, other)) }
	cmtst.8h v3, v5, v5
	cmtst.8h v2, v4, v4
		// src/sim/blockbuilding/state.rs:1132
		let (mut layers, mut attitudes) = {
	strb w5, [sp, #447]
	mov.16b v6, v4
	movi.8h v4, #15
	and.16b v1, v1, v4
	and.16b v0, v0, v4
	stp q0, q1, [sp, #256]
	mov.16b v4, v6
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/masks.rs:194
		Self(mask_impl::Mask::from_int_unchecked(value))
	stp q3, q2, [sp, #448]
	add x25, sp, #432
	add x26, sp, #384
Lloh1506:
	adrp x8, lCPI324_0@PAGE
Lloh1507:
	ldr q0, [x8, lCPI324_0@PAGEOFF]
	str q0, [sp, #240]
	b LBB324_21
LBB324_20:
		// src/sim/blockbuilding/state.rs:1219
		.try_merge(into_block((layers[other_i], attitudes[other_i])), 4)
	lsr w8, w0, #24
	lsr w9, w0, #8
		// src/sim/blockbuilding/state.rs:1226
		layers[layer_i] = merged.layers().to_u16();
	strh w9, [x26, x23, lsl #1]
		// src/sim/blockbuilding/state.rs:1227
		attitudes[layer_i] = merged.attitude();
	strb w8, [x25, x23]
	orr x8, x26, x19, lsl #1
		// src/sim/blockbuilding/state.rs:1229
		layers[other_i] = 0;
	strh wzr, [x8]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:273
		&mut (*slice)[self]
	add x8, sp, #448
		// src/sim/blockbuilding/state.rs:1230
		attitudes[other_i] = ElemId(u8::MAX); // this is technically unnecessary
	mov w9, #255
	strb w9, [x25, x19]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:273
		&mut (*slice)[self]
	bfi x8, x19, #1, #4
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/masks/full_masks.rs:117
		self.0[lane] = if value { T::TRUE } else { T::FALSE }
	strh wzr, [x8]
	ldp q5, q4, [sp, #384]
	mov w24, #1
	ldp q3, q2, [sp, #448]
LBB324_21:
	ext.16b v1, v4, v5, #14
	ext.16b v0, v5, v4, #14
	stp q0, q1, [sp, #208]
	ext.16b v1, v4, v5, #12
	ext.16b v0, v5, v4, #12
	stp q0, q1, [sp, #176]
	ext.16b v1, v4, v5, #10
	ext.16b v0, v5, v4, #10
	stp q0, q1, [sp, #144]
	ext.16b v1, v4, v5, #8
	ext.16b v0, v5, v4, #8
	stp q0, q1, [sp, #112]
	ext.16b v1, v4, v5, #6
	ext.16b v0, v5, v4, #6
	stp q0, q1, [sp, #80]
	ext.16b v1, v4, v5, #4
	ext.16b v0, v5, v4, #4
	stp q0, q1, [sp, #48]
	ext.16b v1, v4, v5, #2
	ext.16b v0, v5, v4, #2
	stp q0, q1, [sp, #16]
	cmlt.8h v0, v3, #0
	cmlt.8h v1, v2, #0
	mov w27, #1
	uzp1.16b v0, v0, v1
	stp q5, q0, [sp, #336]
	str q4, [sp, #320]
	b LBB324_23
LBB324_22:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/range.rs:546
		self.exhausted || !(self.start <= self.end)
	cmp x27, #7
	mov x27, x28
	ldp q4, q5, [sp, #320]
	b.hi LBB324_46
LBB324_23:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/range.rs:1162
		Some(if is_iterating {
	cmp x27, #7
	b.ls LBB324_30
	mov w28, #8
	mov.16b v0, v4
	mov.16b v1, v5
LBB324_25:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/ops.rs:40
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	eor.16b v2, v4, v1
	eor.16b v3, v5, v0
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/simd/cmp/eq.rs:40
		unsafe { Mask::from_int_unchecked(core::intrinsics::simd::simd_ne(self, other)) }
	cmeq.8h v0, v0, #0
	cmeq.8h v1, v1, #0
	uzp1.16b v0, v0, v1
	movi.8h v4, #1
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/simd/num/uint.rs:245
		unsafe { core::intrinsics::simd::simd_cttz(self) }
	sub.8h v1, v3, v4
	bic.16b v1, v1, v3
	clz.8h v1, v1
	movi.8h v5, #16
	sub.8h v1, v5, v1
	sub.8h v4, v2, v4
	bic.16b v4, v4, v2
	clz.8h v4, v4
	sub.8h v4, v5, v4
	ldp q6, q5, [sp, #256]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/ops.rs:40
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	and.16b v4, v6, v4
	and.16b v1, v5, v1
	ldp q6, q5, [sp, #288]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/ops.rs:58
		core::intrinsics::simd::$simd_call(
	ushl.8h v1, v6, v1
	ushl.8h v4, v5, v4
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/ops.rs:40
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	bic.16b v2, v2, v4
	bic.16b v1, v3, v1
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/simd/cmp/eq.rs:33
		unsafe { Mask::from_int_unchecked(core::intrinsics::simd::simd_eq(self, other)) }
	cmeq.8h v1, v1, #0
	cmeq.8h v2, v2, #0
	uzp1.16b v1, v1, v2
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/masks/full_masks.rs:256
		unsafe { Self(core::intrinsics::simd::simd_and(self.0, rhs.0)) }
	bic.16b v0, v1, v0
	ldr q1, [sp, #352]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/../../portable-simd/crates/core_simd/src/masks/full_masks.rs:150
		let bitmask: U = unsafe { core::intrinsics::simd::simd_bitmask(resized) };
	and.16b v0, v0, v1
	ldr q1, [sp, #240]
	and.16b v1, v0, v1
	ext.16b v2, v1, v1, #8
	zip1.16b v1, v1, v2
	addv.8h h1, v1
	str h1, [sp, #382]
		// src/sim/blockbuilding/state.rs:1202
		if can_maybe_merge == 0 {
	umaxv.16b b0, v0
	fmov w8, s0
	tbz w8, #0, LBB324_22
	mov x23, #0
	ldrh w22, [sp, #382]
	b LBB324_28
LBB324_27:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/cmp.rs:1903
		fn lt(&self, other: &Self) -> bool { *self <  *other }
	add x23, x23, #1
		// src/sim/blockbuilding/state.rs:1208
		for layer_i in 0..N {
	cmp x23, #16
	b.eq LBB324_22
LBB324_28:
		// src/sim/blockbuilding/state.rs:1212
		if (can_maybe_merge & (1 << layer_i)) == 0 {
	lsr x8, x22, x23
	tbz w8, #0, LBB324_27
		// src/sim/blockbuilding/state.rs:1215
		let other_i = (offset_i + layer_i) % N;
	add w8, w27, w23
	and x19, x8, #0xf
		// src/sim/blockbuilding/state.rs:1218
		if let Some(merged) = into_block((layers[layer_i], attitudes[layer_i]))
	ldrb w9, [x25, x23]
	ldrh w10, [x26, x23, lsl #1]
		// src/sim/blockbuilding/state.rs:1219
		.try_merge(into_block((layers[other_i], attitudes[other_i])), 4)
	add x11, sp, #384
	bfi x11, x8, #1, #4
	ldrh w8, [x11]
	ldrb w11, [x25, x19]
		// src/sim/blockbuilding/state.rs:1218
		if let Some(merged) = into_block((layers[layer_i], attitudes[layer_i]))
	orr w0, w10, w9, lsl #16
	orr w1, w8, w11, lsl #16
		// src/sim/blockbuilding/state.rs:1219
		.try_merge(into_block((layers[other_i], attitudes[other_i])), 4)
	bl robodoan::sim::blockbuilding::block::Block::try_merge
		// src/sim/blockbuilding/state.rs:1218
		if let Some(merged) = into_block((layers[layer_i], attitudes[layer_i]))
	tbz w0, #0, LBB324_27
	b LBB324_20
LBB324_30:
		// src/sim/blockbuilding/state.rs:1173
		match offset_i {
	cmp x27, #3
	b.le LBB324_35
	cmp x27, #5
	b.gt LBB324_39
	cmp x27, #4
	b.eq LBB324_44
	cmp x27, #5
	b.ne LBB324_72
	mov w28, #6
	ldp q0, q1, [sp, #144]
		// src/sim/blockbuilding/state.rs:1178
		5 => layers.rotate_elements_left::<5>(),
	b LBB324_25
LBB324_35:
		// src/sim/blockbuilding/state.rs:1173
		match offset_i {
	cmp x27, #1
	b.eq LBB324_42
	cmp x27, #2
	b.eq LBB324_43
	cmp x27, #3
	b.ne LBB324_72
	mov w28, #4
	ldp q0, q1, [sp, #80]
		// src/sim/blockbuilding/state.rs:1176
		3 => layers.rotate_elements_left::<3>(),
	b LBB324_25
LBB324_39:
		// src/sim/blockbuilding/state.rs:1173
		match offset_i {
	cmp x27, #6
	b.eq LBB324_45
	cmp x27, #7
	b.ne LBB324_72
	mov w28, #8
	ldp q0, q1, [sp, #208]
	b LBB324_25
LBB324_42:
	mov w28, #2
	ldp q0, q1, [sp, #16]
	b LBB324_25
LBB324_43:
	mov w28, #3
	ldp q0, q1, [sp, #48]
		// src/sim/blockbuilding/state.rs:1175
		2 => layers.rotate_elements_left::<2>(),
	b LBB324_25
LBB324_44:
	mov w28, #5
	ldp q0, q1, [sp, #112]
		// src/sim/blockbuilding/state.rs:1177
		4 => layers.rotate_elements_left::<4>(),
	b LBB324_25
LBB324_45:
	mov w28, #7
	ldp q0, q1, [sp, #176]
		// src/sim/blockbuilding/state.rs:1179
		6 => layers.rotate_elements_left::<6>(),
	b LBB324_25
LBB324_46:
		// src/sim/blockbuilding/state.rs:1260
		if !ever_merged {
	tbz w24, #0, LBB324_54
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:796
		if self < rhs {
	cmp w20, #17
	b.hs LBB324_74
	mov x8, #0
	mov x10, #0
		// src/stackvec.rs:33
		Self {
	movi.2d v0, #0000000000000000
	add x17, sp, #480
	stur q0, [x17, #47]
	str q0, [x17, #32]
	stp q0, q0, [sp, #480]
	add x9, sp, #544
	orr x9, x9, #0x1
	add x11, sp, #432
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2426
		while let Some(x) = self.next() {
	sub x11, x11, #1
	add x12, sp, #384
	ldr x16, [sp, #8]
LBB324_49:
	cmp x10, x20
	csel x14, x10, x20, hi
	mov x15, x10
LBB324_50:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/adapters/zip.rs:306
		if self.index < self.len {
	cmp x14, x15
	b.eq LBB324_55
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/adapters/zip.rs:310
		self.index += 1;
	add x10, x15, #1
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2427
		accum = f(accum, x)?;
	ldrh w13, [x12, x15, lsl #1]
	mov x15, x10
	cbz w13, LBB324_50
	ldrb w14, [x11, x10]
		// src/stackvec.rs:126
		self = self.push(elem)?;
	ldp q0, q1, [sp, #480]
	stp q0, q1, [x9]
	ldr q0, [x17, #32]
	str q0, [x9, #32]
	ldur q0, [x17, #47]
	stur q0, [x9, #47]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:224
		if self < slice.len() {
	cmp x8, #21
	b.eq LBB324_75
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:226
		unsafe { Some(slice_get_unchecked(slice, self)) }
	add x15, x8, x8, lsl #1
	add x15, x9, x15
		// src/stackvec.rs:79
		*self.elems.get_mut(self.len as usize)? = elem;
	strb w14, [x15, #2]
	strh w13, [x15]
		// src/stackvec.rs:80
		self.len += 1;
	add x8, x8, #1
		// src/stackvec.rs:81
		Some(self)
	ldp q0, q1, [x9]
	stp q0, q1, [sp, #480]
	ldr q0, [x9, #32]
	str q0, [x17, #32]
	ldur q0, [x9, #47]
	stur q0, [x17, #47]
		// src/stackvec.rs:125
		for elem in iter {
	b LBB324_49
LBB324_54:
		// src/sim/blockbuilding/state.rs:1261
		return self;
	ldp q0, q1, [x21]
	ldr x8, [sp, #8]
	stp q0, q1, [x8]
	ldp q0, q1, [x21, #32]
	stp q0, q1, [x8, #32]
	b LBB324_56
LBB324_55:
		// src/stackvec.rs:128
		Some(self)
	ldp q0, q1, [sp, #480]
	stur q0, [x16, #1]
	stur q1, [x16, #17]
	ldr q0, [x17, #32]
	stur q0, [x16, #33]
	ldur q0, [x17, #47]
	str q0, [x16, #48]
		// src/sim/blockbuilding/state.rs:1263
		return BlockSet {
	strb w8, [x16]
LBB324_56:
		// src/sim/blockbuilding/state.rs:1283
		}
	sub sp, x29, #80
	.cfi_def_cfa wsp, 96
	ldp x29, x30, [sp, #80]
	ldp x20, x19, [sp, #64]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x26, x25, [sp, #16]
	ldp x28, x27, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	ret
LBB324_57:
	.cfi_restore_state
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x10, x21, #7
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v5 }[2], [x10]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w10, [x21, #9]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #10
	b.eq LBB324_6
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x11, x21, #10
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v5 }[3], [x11]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w11, [x21, #12]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #13
	b.eq LBB324_7
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x12, x21, #13
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v5 }[4], [x12]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w12, [x21, #15]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #16
	b.eq LBB324_8
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x13, x21, #16
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v5 }[5], [x13]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w13, [x21, #18]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #19
	b.eq LBB324_9
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x14, x21, #19
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v5 }[6], [x14]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w14, [x21, #21]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #22
	b.eq LBB324_10
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x15, x21, #22
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v5 }[7], [x15]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w15, [x21, #24]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #25
	b.eq LBB324_11
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ldur h4, [x21, #25]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w16, [x21, #27]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #28
	b.eq LBB324_12
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x17, x21, #28
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v4 }[1], [x17]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w17, [x21, #30]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #31
	b.eq LBB324_13
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x0, x21, #31
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v4 }[2], [x0]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w0, [x21, #33]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #34
	b.eq LBB324_14
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x1, x21, #34
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v4 }[3], [x1]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w1, [x21, #36]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #37
	b.eq LBB324_15
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x2, x21, #37
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v4 }[4], [x2]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w2, [x21, #39]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #40
	b.eq LBB324_16
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x3, x21, #40
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v4 }[5], [x3]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w3, [x21, #42]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #43
	b.eq LBB324_17
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x4, x21, #43
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v4 }[6], [x4]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w4, [x21, #45]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	cmp x5, #46
	b.eq LBB324_18
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:659
		unsafe { NonNull { pointer: intrinsics::offset(self.as_ptr(), count) } }
	add x5, x21, #46
		// src/sim/blockbuilding/state.rs:1136
		layers[i] = block.layers().to_u16();
	ld1.h { v4 }[7], [x5]
		// src/sim/blockbuilding/state.rs:1137
		attitudes[i] = block.attitude();
	ldrb w5, [x21, #48]
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:179
		if ptr == crate::intrinsics::transmute::<$ptr, NonNull<T>>(end_or_len) {
	b LBB324_19
LBB324_71:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:438
		slice_index_fail(self.start, self.end, slice.len())
Lloh1508:
	adrp x3, l_anon.f150cffd4c9c539103ec6afad97ced14.125@PAGE
Lloh1509:
	add x3, x3, l_anon.f150cffd4c9c539103ec6afad97ced14.125@PAGEOFF
	mov x0, #0
	mov x1, x20
	mov w2, #21
	bl core::slice::index::slice_index_fail
LBB324_72:
		// src/sim/blockbuilding/state.rs:1182
		_ => unreachable!(),
Lloh1510:
	adrp x0, l_anon.f150cffd4c9c539103ec6afad97ced14.5@PAGE
Lloh1511:
	add x0, x0, l_anon.f150cffd4c9c539103ec6afad97ced14.5@PAGEOFF
Lloh1512:
	adrp x2, l_anon.f150cffd4c9c539103ec6afad97ced14.158@PAGE
Lloh1513:
	add x2, x2, l_anon.f150cffd4c9c539103ec6afad97ced14.158@PAGEOFF
	mov w1, #40
	bl core::panicking::panic
LBB324_73:
		// src/sim/blockbuilding/state.rs:1115
		16 => assert!((9..=16).contains(&self.blocks.len())),
Lloh1514:
	adrp x0, l_anon.f150cffd4c9c539103ec6afad97ced14.154@PAGE
Lloh1515:
	add x0, x0, l_anon.f150cffd4c9c539103ec6afad97ced14.154@PAGEOFF
Lloh1516:
	adrp x2, l_anon.f150cffd4c9c539103ec6afad97ced14.155@PAGE
Lloh1517:
	add x2, x2, l_anon.f150cffd4c9c539103ec6afad97ced14.155@PAGEOFF
	mov w1, #55
	bl core::panicking::panic
LBB324_74:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/slice/index.rs:438
		slice_index_fail(self.start, self.end, slice.len())
Lloh1518:
	adrp x3, l_anon.f150cffd4c9c539103ec6afad97ced14.157@PAGE
Lloh1519:
	add x3, x3, l_anon.f150cffd4c9c539103ec6afad97ced14.157@PAGEOFF
	mov x0, #0
	mov x1, x20
	mov w2, #16
	bl core::slice::index::slice_index_fail
LBB324_75:
		// ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/option.rs:1015
		None => unwrap_failed(),
Lloh1520:
	adrp x0, l_anon.f150cffd4c9c539103ec6afad97ced14.156@PAGE
Lloh1521:
	add x0, x0, l_anon.f150cffd4c9c539103ec6afad97ced14.156@PAGEOFF
	bl core::option::unwrap_failed
	.loh AdrpAdd	Lloh1504, Lloh1505
	.loh AdrpAdd	Lloh1502, Lloh1503
	.loh AdrpLdr	Lloh1506, Lloh1507
	.loh AdrpAdd	Lloh1508, Lloh1509
	.loh AdrpAdd	Lloh1512, Lloh1513
	.loh AdrpAdd	Lloh1510, Lloh1511
	.loh AdrpAdd	Lloh1516, Lloh1517
	.loh AdrpAdd	Lloh1514, Lloh1515
	.loh AdrpAdd	Lloh1518, Lloh1519
	.loh AdrpAdd	Lloh1520, Lloh1521
