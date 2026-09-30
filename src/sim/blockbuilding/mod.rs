//! Blockbuilding puzzle simulator.

mod block;
mod block_list;
mod block_list_meta;

pub use block::Block;
#[cfg(feature = "dbg_rank_counts")]
pub use block_list::rank_counts;
#[cfg(feature = "dbg_twist_count")]
pub use block_list::twist_count;
pub use block_list::BlockList;
use block_list_meta::BlockListMeta;
