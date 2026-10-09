//! Zero-allocation fixed-capacity buffers and reusable memory pools.

use parking_lot::Mutex;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Fixed-capacity inline stack buffer that performs zero heap allocations.
#[derive(Clone, Copy)]
pub struct ZeroBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> Default for ZeroBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ZeroBuffer<N> {
    pub const fn new() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.len >= N {
            return Err("buffer full");
        }
        self.data[self.len] = byte;
        self.len += 1;
        Ok(())
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), &'static str> {
        if self.len + slice.len() > N {
            return Err("buffer capacity exceeded");
        }
        self.data[self.len..self.len + slice.len()].copy_from_slice(slice);
        self.len += slice.len();
        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.len]
    }
}

impl<const N: usize> Deref for ZeroBuffer<N> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<const N: usize> DerefMut for ZeroBuffer<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<const N: usize> AsRef<[u8]> for ZeroBuffer<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const N: usize> std::io::Write for ZeroBuffer<N> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let avail = N - self.len;
        let to_write = buf.len().min(avail);
        if to_write == 0 && !buf.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "zero buffer is full",
            ));
        }
        self.data[self.len..self.len + to_write].copy_from_slice(&buf[..to_write]);
        self.len += to_write;
        Ok(to_write)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Reusable buffer pool for amortizing heap allocations.
pub struct BufferPool<const CAP: usize> {
    pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl<const CAP: usize> Clone for BufferPool<CAP> {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
        }
    }
}

impl<const CAP: usize> Default for BufferPool<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> BufferPool<CAP> {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Acquire a buffer from the pool, allocating with capacity `CAP` if the pool is empty.
    pub fn acquire(&self) -> PooledBuffer<CAP> {
        let buf = self.pool.lock().pop().unwrap_or_else(|| Vec::with_capacity(CAP));
        PooledBuffer {
            buf: Some(buf),
            pool: Arc::clone(&self.pool),
        }
    }
}

/// RAII guard that recycles the buffer back into the pool upon drop.
pub struct PooledBuffer<const CAP: usize> {
    buf: Option<Vec<u8>>,
    pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl<const CAP: usize> Deref for PooledBuffer<CAP> {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        self.buf.as_ref().unwrap()
    }
}

impl<const CAP: usize> DerefMut for PooledBuffer<CAP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut().unwrap()
    }
}

impl<const CAP: usize> Drop for PooledBuffer<CAP> {
    fn drop(&mut self) {
        if let Some(mut b) = self.buf.take() {
            b.clear();
            self.pool.lock().push(b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_zero_buffer() {
        let mut buf = ZeroBuffer::<32>::new();
        assert_eq!(buf.capacity(), 32);
        assert_eq!(buf.len(), 0);

        buf.write_all(b"blockchain").unwrap();
        assert_eq!(buf.len(), 10);
        assert_eq!(&*buf, b"blockchain");

        buf.clear();
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_buffer_pool_recycling() {
        let pool = BufferPool::<64>::new();
        {
            let mut buf = pool.acquire();
            buf.extend_from_slice(b"recycled data");
            assert_eq!(buf.len(), 13);
        } // returned to pool

        let buf2 = pool.acquire();
        assert_eq!(buf2.len(), 0);
        assert!(buf2.capacity() >= 64);
    }
}
