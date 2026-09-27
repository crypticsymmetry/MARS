//! Micro-benchmark of Hamming kernels on segment-sized slices.
use mars_hv::hv::*;
use mars_hv::Rng;
use std::time::Instant;

fn main() {
    for words in [16usize, 32, 48, 128] {
        let rows: usize = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(200_000);
        let mut data = vec![0u64; rows * words];
        Rng::new(1).fill(&mut data);
        let mut q = vec![0u64; words];
        Rng::new(2).fill(&mut q);
        let run = |name: &str, f: &dyn Fn(&[u64], &[u64]) -> u32| {
            let t = Instant::now();
            let mut s = 0u64;
            for _ in 0..(2_000_000 / rows).max(10) {
                for r in 0..rows {
                    s += f(&q, &data[r * words..(r + 1) * words]) as u64;
                }
            }
            let dt = t.elapsed().as_secs_f64();
            println!("{words:>4} words {name:>10}: {:.2} Gwords/s (chk {s})", (rows * words * (2_000_000 / rows).max(10)) as f64 / dt / 1e9);
        };
        run("scalar4", &|a, b| hamming_words(a, b));
        run("vec-sum", &|a, b| hamming_words_vec(a, b));
        #[cfg(all(target_arch = "x86_64", target_feature = "avx512f", target_feature = "avx512vpopcntdq"))]
        run("avx512", &|a, b| hamming_words_avx512(a, b));
    }
}
