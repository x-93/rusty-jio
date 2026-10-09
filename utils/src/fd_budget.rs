//! File descriptor budget allocation and limits.

use parking_lot::Mutex;
use std::sync::Arc;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("file descriptor budget exhausted: requested {requested}, available {available}")]
pub struct FdBudgetExhausted {
    pub requested: usize,
    pub available: usize,
}

struct FdBudgetInner {
    total: usize,
    used: usize,
}

/// Thread-safe manager for file descriptor allowances.
#[derive(Clone)]
pub struct FdBudget {
    inner: Arc<Mutex<FdBudgetInner>>,
}

impl FdBudget {
    /// Create a new budget with the specified maximum FD count.
    pub fn new(max_fds: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FdBudgetInner {
                total: max_fds,
                used: 0,
            })),
        }
    }

    /// Try to acquire `count` file descriptors. Returns RAII guard on success.
    pub fn try_acquire(&self, count: usize) -> Result<FdGuard, FdBudgetExhausted> {
        let mut guard = self.inner.lock();
        let available = guard.total.saturating_sub(guard.used);
        if count <= available {
            guard.used += count;
            Ok(FdGuard {
                budget: Arc::clone(&self.inner),
                count,
            })
        } else {
            Err(FdBudgetExhausted {
                requested: count,
                available,
            })
        }
    }

    /// Acquire up to `count` file descriptors, or None if budget exceeded.
    pub fn acquire(&self, count: usize) -> Option<FdGuard> {
        self.try_acquire(count).ok()
    }

    pub fn available(&self) -> usize {
        let guard = self.inner.lock();
        guard.total.saturating_sub(guard.used)
    }

    pub fn total(&self) -> usize {
        self.inner.lock().total
    }

    pub fn used(&self) -> usize {
        self.inner.lock().used
    }
}

/// RAII guard that releases file descriptors back to the budget when dropped.
pub struct FdGuard {
    budget: Arc<Mutex<FdBudgetInner>>,
    count: usize,
}

impl FdGuard {
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Drop for FdGuard {
    fn drop(&mut self) {
        let mut guard = self.budget.lock();
        guard.used = guard.used.saturating_sub(self.count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fd_budget_acquire_and_release() {
        let budget = FdBudget::new(10);
        assert_eq!(budget.available(), 10);

        let guard1 = budget.try_acquire(6).unwrap();
        assert_eq!(budget.available(), 4);

        assert!(budget.try_acquire(5).is_err());

        {
            let _guard2 = budget.try_acquire(3).unwrap();
            assert_eq!(budget.available(), 1);
        } // guard2 dropped here

        assert_eq!(budget.available(), 4);
        drop(guard1);
        assert_eq!(budget.available(), 10);
    }
}
