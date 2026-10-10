//! Zero-allocation buffers, bounded channels, memory tracking, and resource budgeting.

pub mod buffer;
pub mod channel;
pub mod fd_budget;
pub mod mem_size;

pub use buffer::{BufferPool, PooledBuffer, ZeroBuffer};
pub use channel::{bounded, ChannelError, ChannelMetrics, ChannelReceiver, ChannelSender};
pub use fd_budget::{FdBudget, FdBudgetExhausted, FdGuard};
pub use mem_size::{MemSize, MemSizeEstimator};
