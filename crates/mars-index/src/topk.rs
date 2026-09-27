//! Bounded top-k collection (largest scores).

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Hit {
    pub id: u32,
    pub score: f32,
}

/// Min-heap entry: the smallest retained score sits on top.
#[derive(Copy, Clone, PartialEq)]
struct Entry(Hit);

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for a min-heap; tie-break on id for determinism (lower id wins).
        other.0.score.total_cmp(&self.0.score).then(self.0.id.cmp(&other.0.id))
    }
}

#[derive(Clone)]
pub struct TopK {
    k: usize,
    heap: BinaryHeap<Entry>,
}

impl TopK {
    pub fn new(k: usize) -> Self {
        TopK { k, heap: BinaryHeap::with_capacity(k + 1) }
    }

    /// Current admission threshold (score a new hit must beat), if full.
    #[inline]
    pub fn threshold(&self) -> Option<f32> {
        if self.heap.len() < self.k {
            None
        } else {
            self.heap.peek().map(|e| e.0.score)
        }
    }

    #[inline]
    pub fn push(&mut self, id: u32, score: f32) {
        if self.k == 0 {
            return;
        }
        if self.heap.len() < self.k {
            self.heap.push(Entry(Hit { id, score }));
        } else if let Some(top) = self.heap.peek() {
            let worse = top.0.score < score || (top.0.score == score && id < top.0.id);
            if worse {
                self.heap.pop();
                self.heap.push(Entry(Hit { id, score }));
            }
        }
    }

    pub fn merge(&mut self, other: TopK) {
        for e in other.heap {
            self.push(e.0.id, e.0.score);
        }
    }

    /// Hits sorted by descending score (ties: ascending id).
    pub fn into_sorted(self) -> Vec<Hit> {
        let mut v: Vec<Hit> = self.heap.into_iter().map(|e| e.0).collect();
        v.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_largest() {
        let mut t = TopK::new(3);
        for (i, s) in [0.1, 0.9, 0.5, 0.7, 0.2, 0.9].iter().enumerate() {
            t.push(i as u32, *s);
        }
        let v = t.into_sorted();
        assert_eq!(v.iter().map(|h| h.id).collect::<Vec<_>>(), vec![1, 5, 3]);
    }
}
