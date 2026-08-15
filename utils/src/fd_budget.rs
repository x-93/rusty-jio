use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FdBudget {
    budget: Arc<AtomicUsize>,
    max_limit: usize,
}

impl FdBudget {
    pub fn new(max_limit: usize) -> Self {
        Self {
            budget: Arc::new(AtomicUsize::new(max_limit)),
            max_limit,
        }
    }

    pub fn acquire(&self) -> bool {
        loop {
            let current = self.budget.load(Ordering::Relaxed);
            if current == 0 {
                return false;
            }
            if self
                .budget
                .compare_exchange_weak(current, current - 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    pub fn release(&self) {
        self.budget.fetch_add(1, Ordering::SeqCst);
    }

    pub fn available(&self) -> usize {
        self.budget.load(Ordering::Relaxed)
    }

    pub fn max_limit(&self) -> usize {
        self.max_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fd_budget() {
        let budget = FdBudget::new(2);
        assert_eq!(budget.available(), 2);
        assert!(budget.acquire());
        assert!(budget.acquire());
        assert!(!budget.acquire());
        budget.release();
        assert!(budget.acquire());
    }
}
