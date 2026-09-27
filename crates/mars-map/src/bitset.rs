//! Minimal fixed-size bitset used for MH sets, descendant closures and nogoods.

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    pub fn new(n: usize) -> Self {
        BitSet { words: vec![0; n.div_ceil(64)] }
    }

    #[inline]
    pub fn insert(&mut self, i: usize) {
        self.words[i / 64] |= 1 << (i % 64);
    }

    #[inline]
    pub fn contains(&self, i: usize) -> bool {
        (self.words[i / 64] >> (i % 64)) & 1 == 1
    }

    #[inline]
    pub fn union_with(&mut self, o: &BitSet) {
        for (a, b) in self.words.iter_mut().zip(&o.words) {
            *a |= *b;
        }
    }

    #[inline]
    pub fn intersects(&self, o: &BitSet) -> bool {
        self.words.iter().zip(&o.words).any(|(a, b)| a & b != 0)
    }

    /// True if every element of `self` is in `o`.
    #[inline]
    pub fn is_subset(&self, o: &BitSet) -> bool {
        self.words.iter().zip(&o.words).all(|(a, b)| a & !b == 0)
    }

    pub fn count(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(wi, &w)| {
            let mut w = w;
            std::iter::from_fn(move || {
                if w == 0 {
                    return None;
                }
                let t = w.trailing_zeros() as usize;
                w &= w - 1;
                Some(wi * 64 + t)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let mut a = BitSet::new(130);
        a.insert(0);
        a.insert(64);
        a.insert(129);
        assert_eq!(a.iter().collect::<Vec<_>>(), vec![0, 64, 129]);
        let mut b = BitSet::new(130);
        b.insert(64);
        assert!(a.intersects(&b));
        assert!(b.is_subset(&a));
        b.union_with(&a);
        assert_eq!(b.count(), 3);
    }
}
