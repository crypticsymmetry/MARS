//! Mode K scan throughput vs measured memory bandwidth (P0 gate G0).

use crate::Args;
use mars_encode::{Layout, Profile, N_CHANNELS};
use mars_hv::Rng;
use mars_index::ModeK;
use rayon::prelude::*;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

fn best_of<T>(reps: usize, mut f: impl FnMut() -> T) -> (f64, T) {
    let mut best = f64::INFINITY;
    let mut out = None;
    for _ in 0..reps {
        let t = Instant::now();
        let r = f();
        best = best.min(t.elapsed().as_secs_f64());
        out = Some(r);
    }
    (best, out.unwrap())
}

pub fn run(args: &Args) -> Result<(), String> {
    let n = args.usize("rows", 1_000_000);
    let reps = args.usize("reps", 3);
    let out_dir = args.str("out", "results/bw");
    let layout = Layout::default();
    let words = layout.total_words();
    eprintln!("[bw] building {n} random fingerprints ({} MiB)", n * words * 8 >> 20);
    let t0 = Instant::now();
    let rows: Vec<Vec<u64>> = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut v = vec![0u64; words];
            Rng::new(i as u64).fill(&mut v);
            v
        })
        .collect();
    let mut idx = ModeK::with_capacity(layout.clone(), n);
    for r in &rows {
        idx.push(r);
    }
    drop(rows);
    eprintln!("[bw] built in {:.2?}", t0.elapsed());

    let profile = Profile::analogy();
    let scorer = idx.scorer(&profile.weights);
    let active_bits: usize = (0..N_CHANNELS).filter(|&c| profile.weights[c] != 0.0).map(|c| layout.dims[c]).sum();
    let bytes_per_scan = (n * active_bits / 8) as f64;

    // Memory read bandwidth probe: parallel XOR-reduce over a buffer of the same size as the scan.
    let probe: Vec<u64> = (0..(bytes_per_scan as usize / 8)).into_par_iter().map(|i| i as u64).collect();
    let (t_probe, x) = best_of(reps, || probe.par_chunks(1 << 16).map(|c| c.iter().fold(0u64, |a, &b| a ^ b)).reduce(|| 0, |a, b| a ^ b));
    std::hint::black_box(x);
    let bw = bytes_per_scan / t_probe / 1e9;
    drop(probe);

    let mut rng = Rng::new(99);
    let queries: Vec<Vec<u64>> = (0..128)
        .map(|_| {
            let mut v = vec![0u64; words];
            rng.fill(&mut v);
            v
        })
        .collect();

    let mut md = String::new();
    writeln!(md, "# Mode K scan throughput\n").unwrap();
    writeln!(md, "Rows: {n}; layout {:?} (analogy profile reads {active_bits} bits/row = {:.0} MiB per scan); threads: {}.\n", layout.dims, bytes_per_scan / (1 << 20) as f64, rayon::current_num_threads()).unwrap();
    writeln!(md, "Memory read bandwidth probe (parallel XOR-reduce, same size): **{bw:.1} GB/s**.\n").unwrap();
    writeln!(md, "| batch Q | time / batch | amortized time / query | effective scan bandwidth (GB/s) | vs probe |").unwrap();
    writeln!(md, "|---|---|---|---|---|").unwrap();
    let mut jrows = Vec::new();
    for q in [1usize, 8, 32, 64, 128] {
        let qs: Vec<&[u64]> = queries[..q].iter().map(|v| v.as_slice()).collect();
        let (t, res) = best_of(reps, || idx.search_batch(&qs, &scorer, 64));
        std::hint::black_box(res);
        let eff = bytes_per_scan * q as f64 / t / 1e9;
        writeln!(md, "| {q} | {:.1} ms | {:.2} ms | {eff:.1} | {:.0}% |", t * 1e3, t * 1e3 / q as f64, 100.0 * eff / bw).unwrap();
        jrows.push(json!({"batch": q, "t_batch_s": t, "t_per_query_s": t / q as f64, "eff_gbps": eff}));
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/bw-{n}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(
        format!("{out_dir}/bw-{n}.json"),
        serde_json::to_string_pretty(&json!({"rows": n, "layout": layout, "probe_gbps": bw, "threads": rayon::current_num_threads(), "batches": jrows})).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
