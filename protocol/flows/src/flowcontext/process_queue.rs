use jio_consensus_core::blockhash::BlockHash;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct ProcessQueue {
    queue: Arc<Mutex<VecDeque<BlockHash>>>,
}

impl ProcessQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn enqueue(&self, hash: BlockHash) {
        self.queue.lock().push_back(hash);
    }

    pub fn dequeue(&self) -> Option<BlockHash> {
        self.queue.lock().pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }
}
