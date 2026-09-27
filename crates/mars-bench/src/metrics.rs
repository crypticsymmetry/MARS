//! Evaluation metrics.

/// ROC-AUC via the Mann–Whitney U statistic, ties counted as 1/2.
/// Returns NaN if either side is empty.
pub fn auc(pos: &[f64], neg: &[f64]) -> f64 {
    if pos.is_empty() || neg.is_empty() {
        return f64::NAN;
    }
    let mut all: Vec<(f64, bool)> = pos.iter().map(|&x| (x, true)).chain(neg.iter().map(|&x| (x, false))).collect();
    all.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    // Average ranks over tie blocks.
    let mut rank_sum_pos = 0.0;
    let mut i = 0;
    while i < all.len() {
        let mut j = i;
        while j + 1 < all.len() && all[j + 1].0 == all[i].0 {
            j += 1;
        }
        let avg_rank = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j {
            if all[k].1 {
                rank_sum_pos += avg_rank;
            }
        }
        i = j + 1;
    }
    let (np, nn) = (pos.len() as f64, neg.len() as f64);
    (rank_sum_pos - np * (np + 1.0) / 2.0) / (np * nn)
}

/// Fraction of pairs with a > b (ties count 1/2).
pub fn win_rate(pairs: &[(f64, f64)]) -> f64 {
    if pairs.is_empty() {
        return f64::NAN;
    }
    pairs.iter().map(|&(a, b)| if a > b { 1.0 } else if a == b { 0.5 } else { 0.0 }).sum::<f64>() / pairs.len() as f64
}

pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

pub fn std(xs: &[f64]) -> f64 {
    if xs.len() < 2 {
        return f64::NAN;
    }
    let m = mean(xs);
    (xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (xs.len() - 1) as f64).sqrt()
}

/// Recall@k: fraction of queries whose target rank (0-based) is < k.
pub fn recall_at(ranks: &[usize], k: usize) -> f64 {
    if ranks.is_empty() {
        return f64::NAN;
    }
    ranks.iter().filter(|&&r| r < k).count() as f64 / ranks.len() as f64
}

pub fn mrr(ranks: &[usize]) -> f64 {
    mean(&ranks.iter().map(|&r| 1.0 / (r as f64 + 1.0)).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auc_cases() {
        assert_eq!(auc(&[3.0, 4.0], &[1.0, 2.0]), 1.0);
        assert_eq!(auc(&[1.0, 2.0], &[3.0, 4.0]), 0.0);
        assert_eq!(auc(&[1.0], &[1.0]), 0.5);
        assert!((auc(&[1.0, 3.0], &[2.0]) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn win_rate_ties() {
        assert_eq!(win_rate(&[(1.0, 0.0), (0.0, 0.0)]), 0.75);
    }
}
