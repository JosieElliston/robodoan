# TODO

- instead of par iter over twists, do par iter over grips, so we can share the BlockSet::split on grip, and maybe do fancy vertical
- cache num_blocks x num_blocks bools for whether we tried to merge these two blocks and it failed (bc of eg attitude mismatch) for future try_merge iters (and also that these blocks haven't changed since then)
- core::hint::assert_unchecked
