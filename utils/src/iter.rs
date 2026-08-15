pub trait IterExtensions: Iterator {
    fn chunks_exact_vec(self, size: usize) -> ChunksExactVec<Self>
    where
        Self: Sized,
    {
        ChunksExactVec { iter: self, size }
    }
}

impl<I: Iterator> IterExtensions for I {}

pub struct ChunksExactVec<I> {
    iter: I,
    size: usize,
}

impl<I: Iterator> Iterator for ChunksExactVec<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            if let Some(item) = self.iter.next() {
                chunk.push(item);
            } else {
                break;
            }
        }
        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunks_exact_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let chunks: Vec<Vec<i32>> = data.into_iter().chunks_exact_vec(2).collect();
        assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
    }
}
