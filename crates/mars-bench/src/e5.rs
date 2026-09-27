//! E5: what does Sparse Distributed Memory add over plain Hamming k-NN?
//!
//! * E5b: SDM-bucket (Mode B) candidate generation vs exhaustive Mode K.
//! * E5a: partial-cue retrieval — does Mode A cleanup help?
//! * E5c: prototype emergence — does an SDM read-out approximate the hidden
//!   template better than explicit k-NN bundling?

use crate::metrics::{mean, recall_at};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_gen::{generate, template_instances, GenConfig, Naming, PerturbOp};
use mars_hv::{Accumulator, HyperVector, Rng};
use mars_index::{build_addresses, AddressMode, ModeK, SdmAuto, SdmBucket};
use mars_rel::{CaseId, CaseKind, Kb};
use rayon::prelude::*;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

fn encode(kb: &Kb, encode: &[CaseId], fit: &[CaseId], sk: &Sketcher) -> Vec<Vec<u64>> {
    let fx = FeatureExtractor::new(kb, FeatureConfig::default());
    let fit_feats: Vec<Features> = fit.par_iter().map(|&c| fx.extract(c)).collect();
    let stats = FeatureStats::fit(fit_feats.iter());
    encode
        .par_iter()
        .map(|&c| {
            let mut f = fx.extract(c);
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f).into_words()
        })
        .collect()
}

fn index_of(layout: &Layout, fps: &[Vec<u64>]) -> ModeK {
    let mut idx = ModeK::with_capacity(layout.clone(), fps.len());
    for f in fps {
        idx.push(f);
    }
    idx
}

pub fn run(args: &Args) -> Result<(), String> {
    let groups = args.usize("groups", 12500);
    let n_queries = args.usize("queries", 1000);
    let seed = args.u64("seed", 1);
    let out_dir = args.str("out", "results/E5");
    let parts = args.str("parts", "a,b,c");
    let t0 = Instant::now();
    let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);
    let layout = sk.layout.clone();
    let profile = Profile::analogy();
    let mut md = String::new();
    let mut j = serde_json::Map::new();
    writeln!(md, "# E5: what does SDM add over plain Hamming k-NN?\n").unwrap();

    // ------------------------------------------------------------ corpus for E5a/E5b
    let need_corpus = parts.contains('a') || parts.contains('b');
    if need_corpus {
        let gcfg = GenConfig { seed, n_groups: groups, naming: Naming::Canonical, distractors: 2, ..Default::default() };
        let mut ds = generate(&gcfg);
        let n = ds.items.len();
        let n_queries = n_queries.min(groups);
        // Partial cues: drop a fraction of the base's facts (E5a).
        let fracs = [0.25, 0.5, 0.75];
        let mut partial: Vec<Vec<CaseId>> = vec![Vec::new(); fracs.len()];
        for qi in 0..n_queries {
            let base = ds.case(ds.groups[qi].base);
            let facts = ds.kb.case(base).facts.clone();
            for (fi, &fr) in fracs.iter().enumerate() {
                let mut rng = Rng::derive(seed ^ 0xE5A, (qi * 10 + fi) as u64);
                let n_drop = ((facts.len() as f64) * fr).round() as usize;
                let drop = rng.sample_indices(facts.len(), n_drop.min(facts.len() - 1));
                let kept: Vec<_> = facts.iter().enumerate().filter(|(i, _)| !drop.contains(i)).map(|(_, &f)| f).collect();
                partial[fi].push(ds.kb.add_case(&format!("g{qi}-partial{fi}"), CaseKind::Query, kept));
            }
        }
        let ds = ds;
        let corpus: Vec<CaseId> = ds.items.iter().map(|it| it.case).collect();
        let fps = encode(&ds.kb, &corpus, &corpus, &sk);
        let idx = index_of(&layout, &fps);
        let s_data = idx.scorer(&profile.weights);
        eprintln!("[e5] corpus {n} encoded ({:.1?})", t0.elapsed());
        let excluded = |qi: usize, id: u32| id as usize == ds.groups[qi].base || id as usize == ds.groups[qi].ls;

        // ---------------- E5b: candidate generation
        if parts.contains('b') {
            let t = Instant::now();
            let qs: Vec<&[u64]> = (0..n_queries).map(|qi| fps[ds.groups[qi].base].as_slice()).collect();
            let full: Vec<Vec<mars_index::Hit>> = qs.chunks(64).flat_map(|c| idx.search_batch(c, &s_data, 66)).collect();
            let modek_ms = t.elapsed().as_secs_f64() * 1e3 / n_queries as f64;
            let rank_in = |qi: usize, hits: &[mars_index::Hit]| -> usize {
                hits.iter().filter(|h| !excluded(qi, h.id)).position(|h| h.id as usize == ds.groups[qi].ta).unwrap_or(usize::MAX / 2)
            };
            let modek_r64 = recall_at(&(0..n_queries).map(|qi| rank_in(qi, &full[qi])).collect::<Vec<_>>(), 64);
            writeln!(md, "## E5b: SDM-bucket candidate generation (corpus {n}, {n_queries} queries, target TA, R@64 after exact re-scoring)\n").unwrap();
            writeln!(md, "Reference: exhaustive Mode K, R@64 = {modek_r64:.3}, {modek_ms:.2} ms/query (batched, 4 threads), 100% of the corpus scored.\n").unwrap();
            writeln!(md, "| H | addresses | A_write | mean / max bucket | A_query | R@64 | corpus scored | ms/query (1 thread) |").unwrap();
            writeln!(md, "|---|---|---|---|---|---|---|---|").unwrap();
            let mut rows = Vec::new();
            for &h in &[1024usize, 4096] {
                for (mode_name, mode) in [("random", AddressMode::Random), ("data", AddressMode::DataSample), ("k-means(2)", AddressMode::KMeans { iters: 2 })] {
                    let addrs = build_addresses(&layout, &fps, h, mode, &profile.weights, seed ^ 0xADD);
                    for &aw in &[1usize, 3] {
                        let b = SdmBucket::build(&layout, addrs.clone(), &fps, &profile.weights, aw);
                        let (mean_load, max_load) = b.load_stats();
                        let sa = b.address_scorer(&profile.weights);
                        for &aq in &[1usize, 4, 16, 64] {
                            let t = Instant::now();
                            let res: Vec<(usize, usize)> = (0..n_queries)
                                .into_par_iter()
                                .map(|qi| {
                                    let (hits, nc) = b.search(qs[qi], &idx, &s_data, &sa, aq, 66);
                                    (rank_in(qi, &hits), nc)
                                })
                                .collect();
                            let ms = t.elapsed().as_secs_f64() * 1e3 * rayon::current_num_threads() as f64 / n_queries as f64;
                            let r64 = recall_at(&res.iter().map(|x| x.0).collect::<Vec<_>>(), 64);
                            let frac = mean(&res.iter().map(|x| x.1 as f64 / n as f64).collect::<Vec<_>>());
                            writeln!(md, "| {h} | {mode_name} | {aw} | {mean_load:.1} / {max_load} | {aq} | {r64:.3} | {:.2}% | {ms:.2} |", frac * 100.0).unwrap();
                            rows.push(json!({"h": h, "addr": mode_name, "a_write": aw, "a_query": aq, "r64": r64, "frac_scored": frac, "ms": ms, "mean_load": mean_load, "max_load": max_load}));
                        }
                    }
                }
                eprintln!("[e5b] H={h} done ({:.1?})", t0.elapsed());
            }
            j.insert("e5b".into(), json!({"modek_r64": modek_r64, "modek_ms": modek_ms, "rows": rows}));
        }

        // ---------------- E5a: partial cues
        if parts.contains('a') {
            let pfps: Vec<Vec<Vec<u64>>> = partial.iter().map(|cs| encode(&ds.kb, cs, &corpus, &sk)).collect();
            writeln!(md, "\n## E5a: partial-cue retrieval (corpus {n}; cue = base with a fraction of facts removed)\n").unwrap();
            writeln!(md, "*Self* = top-1 is the full base (or its literal twin LS). *Analogy* = rank of TA with base and LS excluded. Mode A: autoassociative SDM storing all {n} fingerprints; cleanup = iterated read-out used as the query. kNN-bundle = majority of the cue's top-10 Mode K neighbours (explicit cleanup baseline).\n").unwrap();
            writeln!(md, "| removed | method | self@1 | analogy R@1 | analogy R@10 |").unwrap();
            writeln!(md, "|---|---|---|---|---|").unwrap();
            let mut sdms: Vec<(String, SdmAuto)> = Vec::new();
            for &(h, a) in &[(8192usize, 16usize), (32768, 64)] {
                let t = Instant::now();
                let addrs = build_addresses(&layout, &fps, h, AddressMode::DataSample, &profile.weights, seed ^ 0xA0);
                let mut sdm = SdmAuto::new(&layout, addrs, a, seed);
                let s = sdm.scorer(&profile.weights);
                let pats: Vec<&[u64]> = fps.iter().map(|v| v.as_slice()).collect();
                sdm.write_batch(&pats, &s);
                eprintln!("[e5a] SDM H={h} A={a} written ({:.1?}, {} MiB)", t.elapsed(), sdm.memory_bytes() >> 20);
                sdms.push((format!("Mode A cleanup (H={h}, A={a}, 3 iters)"), sdm));
            }
            let dim = layout.total_bits();
            let tie = HyperVector::random(dim, seed ^ 0x71E);
            let mut rows = Vec::new();
            for (fi, fr) in fracs.iter().enumerate() {
                let cues = &pfps[fi];
                let mut methods: Vec<(String, Vec<Vec<u64>>)> = vec![("Mode K on the raw cue".into(), cues.clone())];
                let knn: Vec<Vec<u64>> = cues
                    .par_iter()
                    .map(|c| {
                        let hits = idx.search_serial(c, &s_data, 10);
                        let mut acc = Accumulator::new(dim);
                        for h in hits {
                            acc.add_words(&fps[h.id as usize], 1);
                        }
                        acc.threshold(&tie).into_words()
                    })
                    .collect();
                methods.push(("kNN-bundle(10) cleanup".into(), knn));
                for (name, sdm) in &sdms {
                    let s = sdm.scorer(&profile.weights);
                    methods.push((name.clone(), cues.par_iter().map(|c| sdm.recall(c, &s, 3)).collect()));
                }
                for (name, qv) in methods {
                    let res: Vec<(bool, usize)> = (0..n_queries)
                        .into_par_iter()
                        .map(|qi| {
                            let g = &ds.groups[qi];
                            let hits = idx.search_serial(&qv[qi], &s_data, 16);
                            let self1 = hits.first().map(|h| h.id as usize == g.base || h.id as usize == g.ls).unwrap_or(false);
                            let r = hits.iter().filter(|h| !excluded(qi, h.id)).position(|h| h.id as usize == g.ta).unwrap_or(usize::MAX / 2);
                            (self1, r)
                        })
                        .collect();
                    let self1 = mean(&res.iter().map(|x| x.0 as u8 as f64).collect::<Vec<_>>());
                    let ranks: Vec<usize> = res.iter().map(|x| x.1).collect();
                    writeln!(md, "| {:.0}% | {name} | {self1:.3} | {:.3} | {:.3} |", fr * 100.0, recall_at(&ranks, 1), recall_at(&ranks, 10)).unwrap();
                    rows.push(json!({"removed": fr, "method": name, "self1": self1, "analogy_r1": recall_at(&ranks, 1), "analogy_r10": recall_at(&ranks, 10)}));
                }
            }
            j.insert("e5a".into(), json!(rows));
            eprintln!("[e5a] done ({:.1?})", t0.elapsed());
        }
    }

    // ------------------------------------------------------------ E5c: prototypes
    if parts.contains('c') {
        let n_t = args.usize("templates", 100);
        let per = args.usize("per", 100);
        let held = 5usize;
        let sev = args.usize("severity", 1);
        let cfg = GenConfig { seed, naming: Naming::Canonical, distractors: 2, perturb_ops: PerturbOp::ALL.to_vec(), severity: sev, ..Default::default() };
        let inst = template_instances(&cfg, n_t, per + held);
        let all: Vec<CaseId> = inst.cases.iter().map(|x| x.0).collect();
        let fps_all = encode(&inst.kb, &all, &all, &sk);
        let mut clean = vec![0usize; n_t];
        let mut stored = Vec::new();
        let mut queries = Vec::new();
        let mut seen = vec![0usize; n_t];
        for (i, &(_, t, is_clean)) in inst.cases.iter().enumerate() {
            if is_clean {
                clean[t] = i;
            } else {
                seen[t] += 1;
                if seen[t] <= per {
                    stored.push(i);
                } else {
                    queries.push(i);
                }
            }
        }
        let sfps: Vec<Vec<u64>> = stored.iter().map(|&i| fps_all[i].clone()).collect();
        let sidx = index_of(&layout, &sfps);
        let s_st = sidx.scorer(&profile.weights);
        let cidx = index_of(&layout, &clean.iter().map(|&i| fps_all[i].clone()).collect::<Vec<_>>());
        let s_c = cidx.scorer(&profile.weights);
        let dim = layout.total_bits();
        let tie = HyperVector::random(dim, seed ^ 0x7C);
        let sim = |a: &[u64], b: &[u64]| -> f64 { profile.score(&mars_encode::channel_sims(&layout, a, b)) };
        let bundle = |ids: &[u32]| -> Vec<u64> {
            let mut acc = Accumulator::new(dim);
            for &i in ids {
                acc.add_words(&sfps[i as usize], 1);
            }
            acc.threshold(&tie).into_words()
        };
        let mut methods: Vec<(String, Vec<Vec<u64>>)> = Vec::new();
        methods.push(("cue itself".into(), queries.iter().map(|&q| fps_all[q].clone()).collect()));
        methods.push(("nearest stored neighbour".into(), queries.par_iter().map(|&q| sfps[sidx.search_serial(&fps_all[q], &s_st, 1)[0].id as usize].clone()).collect()));
        for k in [10usize, 50] {
            methods.push((format!("kNN-bundle({k})"), queries.par_iter().map(|&q| bundle(&sidx.search_serial(&fps_all[q], &s_st, k).iter().map(|h| h.id).collect::<Vec<_>>())).collect()));
        }
        for &(h, a) in &[(2048usize, 16usize), (2048, 64), (8192, 64)] {
            let addrs = build_addresses(&layout, &sfps, h, AddressMode::DataSample, &profile.weights, seed ^ 0xC0);
            let mut sdm = SdmAuto::new(&layout, addrs, a, seed);
            let s = sdm.scorer(&profile.weights);
            sdm.write_batch(&sfps.iter().map(|v| v.as_slice()).collect::<Vec<_>>(), &s);
            for iters in [1usize, 3] {
                methods.push((format!("SDM read-out (H={h}, A={a}, {iters} iter)"), queries.par_iter().map(|&q| sdm.recall(&fps_all[q], &s, iters)).collect()));
            }
        }
        writeln!(md, "\n## E5c: prototype emergence ({n_t} hidden templates × {per} stored noisy instances; {} held-out cues; perturbation severity {sev}, 2 distractors)\n", queries.len()).unwrap();
        writeln!(md, "*Sim to prototype* = analogy-profile similarity of the method's output to the template's clean instance (never stored). *Template acc* = nearest clean prototype is the right template.\n").unwrap();
        writeln!(md, "| method | sim to prototype | template acc |").unwrap();
        writeln!(md, "|---|---|---|").unwrap();
        let mut rows = Vec::new();
        for (name, outs) in &methods {
            let sims: Vec<f64> = queries.iter().zip(outs).map(|(&q, o)| sim(o, &fps_all[clean[inst.cases[q].1]])).collect();
            let acc: Vec<f64> = queries.iter().zip(outs).map(|(&q, o)| (cidx.search_serial(o, &s_c, 1)[0].id as usize == inst.cases[q].1) as u8 as f64).collect();
            writeln!(md, "| {name} | {:.3} | {:.3} |", mean(&sims), mean(&acc)).unwrap();
            rows.push(json!({"method": name, "sim_to_prototype": mean(&sims), "template_acc": mean(&acc)}));
        }
        j.insert("e5c".into(), json!(rows));
        eprintln!("[e5c] done ({:.1?})", t0.elapsed());
    }

    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", "main");
    std::fs::write(format!("{out_dir}/E5-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E5-{tag}.json"), serde_json::to_string_pretty(&serde_json::Value::Object(j)).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
