//! EV1 (pre-registered, docs/PREREGISTRATION.md): knowledge-graph link
//! prediction on FB15k-237 / WN18RR under the standard filtered protocol, with
//! the frozen v0.1 components (encoder, Mode K, mapper, transfer typing,
//! identity index).
//!
//! Memory: one case per entity from *training* triples, (r e x) and (r⁻¹ e y),
//! capped at `cap` facts (rarest relations first). A query (e, r, ?) uses e's
//! case; analogues are the other entities (fingerprint shortlist re-ranked by
//! fac·FAC + (1−fac)·FP, optionally fused with the identity channel);
//! candidates are the objects of projected inferences (r e ?). Ranked by raw
//! evidence (Σ fused score) and by learned transfer reliability × evidence,
//! reliability learned from feedback on validation queries only.
//!
//! Modes:
//! * `--split valid --sample N --profile a,b --identity x,y --k m,n`: the
//!   validation grid (raw ranking, MRR), used to choose settings;
//! * `--split test` with single settings: learn reliability on all validation
//!   queries (top-`feedback` suggestions), freeze it, evaluate every test triple
//!   in both directions. Unproposed entities get the expected rank under random
//!   ordering after the proposed ones.

use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_engine::identity::IdentityIndex;
use mars_engine::transfer::{transfer_keys, TransferStats};
use mars_hv::Rng;
use mars_index::ModeK;
use mars_map::{MapConfig, Mapper, Proj};
use mars_rel::{CaseId, CaseKind, Kb, PredKind, Sym, Term};
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

type Triple = (u32, u32, u32);

fn load(path: &str, ents: &mut FxHashMap<String, u32>, rels: &mut FxHashMap<String, u32>) -> Result<Vec<Triple>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let mut out = Vec::new();
    for line in text.lines() {
        let p: Vec<&str> = line.split('\t').map(|s| s.trim()).collect();
        if p.len() != 3 {
            continue;
        }
        let id = |m: &mut FxHashMap<String, u32>, s: &str| {
            let n = m.len() as u32;
            *m.entry(s.to_string()).or_insert(n)
        };
        let h = id(ents, p[0]);
        let r = id(rels, p[1]);
        let t = id(ents, p[2]);
        out.push((h, r, t));
    }
    Ok(out)
}

/// One link-prediction query: entity, directed relation (r or r + R for the inverse), answer.
#[derive(Clone, Copy)]
struct Q {
    e: u32,
    rel: u32,
    ans: u32,
}

/// One candidate object with its evidence: (object, is substitution, Σ weight, transfer keys).
type Cand = (u32, bool, f64, Vec<String>);
/// Per-object scores (raw, learned) and the merged inferences with their learned scores.
type Scored = (FxHashMap<u32, f64>, FxHashMap<u32, f64>, Vec<Cand>);

/// Expected 1-based rank of `ans` among the non-filtered entities, given scores for the
/// proposed ones (ties and unproposed entities ordered at random).
fn filtered_rank(scores: &FxHashMap<u32, f64>, ans: u32, filt: &FxHashSet<u32>, n_ent: usize) -> f64 {
    let s_ans = scores.get(&ans).copied();
    let mut p = 0usize;
    let (mut gt, mut eq) = (0usize, 0usize);
    for (&y, &s) in scores {
        if y != ans && filt.contains(&y) {
            continue;
        }
        p += 1;
        if y == ans {
            continue;
        }
        if let Some(sa) = s_ans {
            if s > sa {
                gt += 1;
            } else if s == sa {
                eq += 1;
            }
        }
    }
    match s_ans {
        Some(_) => 1.0 + gt as f64 + eq as f64 / 2.0,
        None => {
            let filt_unproposed = filt.iter().filter(|y| **y != ans && !scores.contains_key(y)).count();
            let u = n_ent - p - filt_unproposed; // unproposed, non-filtered, including ans
            p as f64 + (u as f64 + 1.0) / 2.0
        }
    }
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/external/kgc/FB15k-237");
    let name = args.str("name", dir.rsplit('/').next().unwrap_or("kg"));
    let split = args.str("split", "valid");
    let sample = args.usize("sample", 0);
    let cap = args.usize("cap", 100);
    let shortlist = args.usize("shortlist", 50);
    let fac_w = args.f64("fac", 0.5);
    let fb_top = args.usize("feedback", 3);
    let seed = args.u64("seed", 1);
    let out_dir = args.str("out", "results/EV1");
    let profiles: Vec<String> = args.str("profile", "analogy").split(',').map(String::from).collect();
    let lambdas: Vec<f64> = args.str("identity", "0").split(',').map(|x| x.parse().unwrap_or(0.0)).collect();
    let ks: Vec<usize> = args.str("k", "10").split(',').map(|x| x.parse().unwrap_or(10)).collect();
    let k_max = *ks.iter().max().unwrap();
    let t0 = Instant::now();

    // Data.
    let (mut ents, mut rels) = (FxHashMap::default(), FxHashMap::default());
    let train = load(&format!("{dir}/train.txt"), &mut ents, &mut rels)?;
    let valid = load(&format!("{dir}/valid.txt"), &mut ents, &mut rels)?;
    let test = load(&format!("{dir}/test.txt"), &mut ents, &mut rels)?;
    let (n, nr) = (ents.len(), rels.len() as u32);
    // Known answers per (entity, directed relation) over all splits (filtering).
    let mut known: FxHashMap<(u32, u32), FxHashSet<u32>> = FxHashMap::default();
    for &(h, r, t) in train.iter().chain(&valid).chain(&test) {
        known.entry((h, r)).or_default().insert(t);
        known.entry((t, r + nr)).or_default().insert(h);
    }
    // Training adjacency (directed facts) and relation frequencies.
    let mut adj: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n];
    let mut freq = vec![0u32; 2 * nr as usize];
    for &(h, r, t) in &train {
        adj[h as usize].push((r, t));
        adj[t as usize].push((r + nr, h));
        freq[r as usize] += 1;
        freq[(r + nr) as usize] += 1;
    }
    let mut und: Vec<Vec<u32>> = adj.iter().map(|v| v.iter().map(|x| x.1).collect()).collect();
    for v in &mut und {
        v.sort_unstable();
        v.dedup();
    }
    // Popularity: answers per directed relation in train.
    let mut pop: Vec<FxHashMap<u32, f64>> = vec![FxHashMap::default(); 2 * nr as usize];
    for &(h, r, t) in &train {
        *pop[r as usize].entry(t).or_insert(0.0) += 1.0;
        *pop[(r + nr) as usize].entry(h).or_insert(0.0) += 1.0;
    }

    // Knowledge base: opaque ids, one case per entity (CaseId == entity index).
    let mut kb = Kb::new();
    let rel_sym: Vec<Sym> = (0..2 * nr).map(|j| kb.declare(&format!("r{j}"), Some(2), PredKind::Relation, false, &[])).collect();
    let ent_sym: Vec<Sym> = (0..n).map(|i| kb.sym(&format!("e{i}"))).collect();
    for i in 0..n {
        let mut f = adj[i].clone();
        f.sort_by_key(|&(r, x)| (freq[r as usize], r, x));
        f.dedup();
        f.truncate(cap);
        let facts: Vec<_> = f.iter().map(|&(r, x)| kb.intern_expr(rel_sym[r as usize], [Term::Ent(ent_sym[i]), Term::Ent(ent_sym[x as usize])])).collect();
        let c = kb.add_case(&format!("c{i}"), CaseKind::Episode, facts);
        debug_assert_eq!(c.0 as usize, i);
    }
    eprintln!("[ev1] {name}: {n} entities, {nr} relations, {} / {} / {} triples; cases built ({:.1?})", train.len(), valid.len(), test.len(), t0.elapsed());

    // Fingerprints (IDF epoch over the training-built cases) and index.
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let raw: Vec<Features> = (0..n).into_par_iter().map(|i| fx.extract(CaseId(i as u32))).collect();
    let stats = FeatureStats::fit(raw.iter());
    let layout = Layout::default();
    let sk = Sketcher::new(layout.clone(), 0xF1);
    let fps: Vec<Vec<u64>> = raw
        .into_par_iter()
        .map(|mut f| {
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f).into_words()
        })
        .collect();
    let mut index = ModeK::with_capacity(layout, n);
    for f in &fps {
        index.push(f);
    }
    let ident = lambdas.iter().any(|&l| l > 0.0).then(|| IdentityIndex::new(&kb, IdentityIndex::fit(&kb, (0..n).map(|i| CaseId(i as u32)))));
    let alive = vec![true; n];
    let mapper = Mapper::new(&kb, MapConfig::default());
    let self_s: Vec<f64> = (0..n).into_par_iter().map(|i| mapper.score(CaseId(i as u32), CaseId(i as u32)) as f64).collect();
    eprintln!("[ev1] encoded and indexed ({:.1?})", t0.elapsed());

    let queries_of = |triples: &[Triple], sample: usize, salt: u64| -> Vec<Q> {
        let mut idx: Vec<usize> = (0..triples.len()).collect();
        Rng::new(seed ^ salt).shuffle(&mut idx);
        if sample > 0 {
            idx.truncate(sample);
        }
        idx.iter().flat_map(|&i| {
            let (h, r, t) = triples[i];
            [Q { e: h, rel: r, ans: t }, Q { e: t, rel: r + nr, ans: h }]
        }).collect()
    };

    // Analogues and candidate votes for a batch of queries, for one (profile, λ).
    let votes_for = |qs: &[Q], prof: &Profile, lam: f64| -> Vec<Vec<(usize, Vec<Cand>)>> {
        let scorer = index.scorer(&prof.weights);
        let mut out: Vec<Vec<(usize, Vec<Cand>)>> = Vec::with_capacity(qs.len());
        for chunk in qs.chunks(256) {
            let qv: Vec<&[u64]> = chunk.iter().map(|q| fps[q.e as usize].as_slice()).collect();
            let hits = index.search_batch(&qv, &scorer, shortlist + 1);
            let res: Vec<Vec<(usize, Vec<Cand>)>> = chunk
                .par_iter()
                .zip(hits.par_iter())
                .map(|(q, h)| {
                    let qc = CaseId(q.e);
                    let mut cand: Vec<(u32, f64)> = h.iter().filter(|x| x.id != q.e).map(|x| (x.id, x.score as f64)).collect();
                    if let (Some(ix), true) = (&ident, lam > 0.0) {
                        for (c, _) in ix.top(qc, shortlist, &alive) {
                            if !cand.iter().any(|x| x.0 == c.0) {
                                cand.push((c.0, index.score_pair(&scorer, &fps[q.e as usize], &fps[c.0 as usize]) as f64));
                            }
                        }
                    }
                    let mut ranked: Vec<(u32, f64)> = cand
                        .into_iter()
                        .map(|(c, fp)| {
                            let s = mapper.score(CaseId(c), qc) as f64;
                            let fac = if s == 0.0 || self_s[c as usize] == 0.0 || self_s[q.e as usize] == 0.0 { 0.0 } else { (s / (self_s[q.e as usize] * self_s[c as usize]).sqrt()).min(1.0) };
                            let fused = fac_w * fac + (1.0 - fac_w) * fp;
                            let sc = match (&ident, lam > 0.0) {
                                (Some(ix), true) => (1.0 - lam) * fused + lam * ix.score(qc, CaseId(c)),
                                _ => fused,
                            };
                            (c, sc)
                        })
                        .collect();
                    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                    ranked.truncate(k_max);
                    let r = rel_sym[q.rel as usize];
                    ranked
                        .iter()
                        .enumerate()
                        .map(|(pos, &(c, w))| {
                            let Some(m) = mapper.best(CaseId(c), qc) else { return (pos, Vec::new()) };
                            let v: Vec<Cand> = m
                                .inferences
                                .iter()
                                .filter(|i| kb.expr(i.base_fact).functor == r && kb.expr(i.base_fact).args.first() == Some(&Term::Ent(ent_sym[c as usize])))
                                .filter_map(|i| match &i.projected {
                                    Proj::Expr { args, .. } if args.len() == 2 && args[0] == Proj::Target(Term::Ent(ent_sym[q.e as usize])) => {
                                        let (y, sub) = match args[1] {
                                            Proj::Target(Term::Ent(y)) => (y, true),
                                            Proj::Skolem(y) => (y, false),
                                            _ => return None,
                                        };
                                        let yi: u32 = kb.name(y)[1..].parse().ok()?;
                                        Some((yi, sub, w.max(1e-3), transfer_keys(&kb, qc, &i.projected)))
                                    }
                                    _ => None,
                                })
                                .collect();
                            (pos, v)
                        })
                        .collect()
                })
                .collect();
            out.extend(res);
        }
        out
    };
    // Scores per object from the first k analogues: (raw Σ weight, learned Σ weight × reliability).
    let score = |v: &[(usize, Vec<Cand>)], k: usize, stats: &TransferStats| -> Scored {
        // Merge by text (object, substitution flag), as Engine::infer does.
        let mut by: FxHashMap<(u32, bool), (f64, Vec<String>)> = FxHashMap::default();
        for (_, cs) in v.iter().filter(|x| x.0 < k) {
            for (y, sub, w, keys) in cs {
                let e = by.entry((*y, *sub)).or_insert((0.0, keys.clone()));
                e.0 += w;
            }
        }
        let (mut raw, mut learned) = (FxHashMap::default(), FxHashMap::default());
        let mut items = Vec::new();
        for ((y, sub), (w, keys)) in by {
            let rel = stats.reliability(&keys);
            *raw.entry(y).or_insert(0.0) += w;
            *learned.entry(y).or_insert(0.0) += w * rel;
            items.push((y, sub, w * rel, keys));
        }
        (raw, learned, items)
    };
    let filt_of = |q: &Q| -> FxHashSet<u32> {
        let mut f = known.get(&(q.e, q.rel)).cloned().unwrap_or_default();
        f.remove(&q.ans);
        f
    };

    let mut md = String::new();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    if split == "valid" {
        // Validation grid: raw ranking, MRR.
        let qs = queries_of(&valid, sample, 0xE71);
        let filts: Vec<FxHashSet<u32>> = qs.iter().map(filt_of).collect();
        writeln!(md, "# EV1 validation grid: {name}\n\n{} validation queries ({} triples × 2 directions, seed {seed}); raw ranking (Σ fused analogue score); cap {cap}, shortlist {shortlist}, FAC weight {fac_w}.\n\n| profile | identity λ | k | MRR | Hits@1 | Hits@10 |\n|---|---|---|---|---|---|", qs.len(), qs.len() / 2).unwrap();
        let mut rows = Vec::new();
        let none = TransferStats::default();
        for p in &profiles {
            let prof = match p.as_str() {
                "literal" => Profile::literal(),
                "surface" => Profile::surface_only(),
                _ => Profile::analogy(),
            };
            for &lam in &lambdas {
                let votes = votes_for(&qs, &prof, lam);
                for &k in &ks {
                    let ranks: Vec<f64> = votes.par_iter().zip(qs.par_iter()).zip(filts.par_iter()).map(|((v, q), f)| filtered_rank(&score(v, k, &none).0, q.ans, f, n)).collect();
                    let mrr = ranks.iter().map(|r| 1.0 / r).sum::<f64>() / ranks.len() as f64;
                    let h = |c: f64| ranks.iter().filter(|&&r| r <= c).count() as f64 / ranks.len() as f64;
                    writeln!(md, "| {p} | {lam} | {k} | {mrr:.4} | {:.4} | {:.4} |", h(1.0), h(10.0)).unwrap();
                    rows.push(json!({"profile": p, "identity": lam, "k": k, "mrr": mrr, "hits1": h(1.0), "hits10": h(10.0)}));
                    eprintln!("[ev1] grid {p} λ={lam} k={k}: MRR {mrr:.4} ({:.1?})", t0.elapsed());
                }
            }
        }
        let best = rows.iter().max_by(|a, b| a["mrr"].as_f64().unwrap().total_cmp(&b["mrr"].as_f64().unwrap())).cloned();
        writeln!(md, "\nChosen (max MRR): {}", best.as_ref().map(|b| b.to_string()).unwrap_or_default()).unwrap();
        std::fs::write(format!("{out_dir}/EV1-{name}-valid-grid.md"), &md).map_err(|e| e.to_string())?;
        std::fs::write(format!("{out_dir}/EV1-{name}-valid-grid.json"), serde_json::to_string_pretty(&json!({"dataset": name, "seed": seed, "sample_triples": sample, "cap": cap, "shortlist": shortlist, "fac": fac_w, "grid": rows, "chosen": best})).unwrap()).map_err(|e| e.to_string())?;
        print!("{md}");
        return Ok(());
    }

    // Test: single settings.
    let (p, lam, k) = (profiles[0].clone(), lambdas[0], ks[0]);
    let prof = match p.as_str() {
        "literal" => Profile::literal(),
        "surface" => Profile::surface_only(),
        _ => Profile::analogy(),
    };
    // 1. Learn transfer reliability from feedback on all validation queries (seeded order).
    let vq = queries_of(&valid, 0, 0xFEED);
    let vvotes = votes_for(&vq, &prof, lam);
    let mut stats_t = TransferStats::default();
    let mut n_fb = 0usize;
    for (v, q) in vvotes.iter().zip(&vq) {
        let (_, _, mut items) = score(v, k, &stats_t);
        items.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));
        for (y, _, _, keys) in items.into_iter().take(fb_top) {
            stats_t.record(&keys, y == q.ans);
            n_fb += 1;
        }
    }
    eprintln!("[ev1] learned reliability from {n_fb} validation feedback events ({:.1?})", t0.elapsed());
    // 2. Test, reliability frozen.
    let tq = queries_of(&test, 0, 0x7E57);
    let tvotes = votes_for(&tq, &prof, lam);
    let per: Vec<(f64, f64, f64, bool)> = tvotes
        .par_iter()
        .zip(tq.par_iter())
        .map(|(v, q)| {
            let f = filt_of(q);
            let (raw, learned, _) = score(v, k, &stats_t);
            let pr = &pop[q.rel as usize];
            // Popularity: expected rank by train frequency (ties random).
            let s_ans = pr.get(&q.ans).copied().unwrap_or(0.0);
            let gt = pr.iter().filter(|(y, s)| **s > s_ans && !f.contains(y)).count();
            let eq_nonzero = pr.iter().filter(|(y, s)| **s == s_ans && **y != q.ans && !f.contains(y)).count();
            let eq = if s_ans == 0.0 {
                let zero_total = n - pr.len();
                let filt_zero = f.iter().filter(|y| !pr.contains_key(y)).count();
                zero_total - 1 - filt_zero
            } else {
                eq_nonzero
            };
            let rank_pop = 1.0 + gt as f64 + eq as f64 / 2.0;
            let (a, b) = (&und[q.e as usize], &und[q.ans as usize]);
            let in2 = a.binary_search(&q.ans).is_ok() || { let (mut i, mut j) = (0, 0); let mut hit = false; while i < a.len() && j < b.len() { if a[i] == b[j] { hit = true; break } else if a[i] < b[j] { i += 1 } else { j += 1 } } hit };
            (filtered_rank(&learned, q.ans, &f, n), filtered_rank(&raw, q.ans, &f, n), rank_pop, in2)
        })
        .collect();
    let metrics = |sel: &dyn Fn(usize) -> bool, m: usize| -> [f64; 4] {
        let r: Vec<f64> = per.iter().enumerate().filter(|(i, _)| sel(*i)).map(|(_, x)| [x.0, x.1, x.2][m]).collect();
        let nn = r.len().max(1) as f64;
        [r.iter().map(|x| 1.0 / x).sum::<f64>() / nn, r.iter().filter(|&&x| x <= 1.0).count() as f64 / nn, r.iter().filter(|&&x| x <= 3.0).count() as f64 / nn, r.iter().filter(|&&x| x <= 10.0).count() as f64 / nn]
    };
    let all = |_: usize| true;
    let near = |i: usize| per[i].3;
    let far = |i: usize| !per[i].3;
    let n_near = per.iter().filter(|x| x.3).count();
    writeln!(md, "# EV1 test: {name}\n\nFrozen v0.1 components. Settings (chosen on validation): profile {p}, identity λ {lam}, k {k}; cap {cap}, shortlist {shortlist}, FAC weight {fac_w}. Reliability learned from {n_fb} feedback events on all {} validation queries, then frozen. {} test queries ({} triples × 2 directions); filtered ranks over {n} entities; unproposed entities at their expected random rank. Runtime {:.1?}.\n", vq.len(), tq.len(), test.len(), t0.elapsed()).unwrap();
    writeln!(md, "| ranking | subset | n | MRR | Hits@1 | Hits@3 | Hits@10 |\n|---|---|---|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    for (mi, mname) in [(0usize, "MARS learned reliability"), (1, "MARS raw"), (2, "relation popularity")] {
        for (sname, sel, cnt) in [("all", &all as &dyn Fn(usize) -> bool, per.len()), ("answer within 2 hops", &near, n_near), ("answer farther", &far, per.len() - n_near)] {
            let m = metrics(sel, mi);
            writeln!(md, "| {mname} | {sname} | {cnt} | {:.4} | {:.4} | {:.4} | {:.4} |", m[0], m[1], m[2], m[3]).unwrap();
            rows.push(json!({"ranking": mname, "subset": sname, "n": cnt, "mrr": m[0], "hits1": m[1], "hits3": m[2], "hits10": m[3]}));
        }
    }
    let tag = format!("EV1-{name}-test");
    std::fs::write(format!("{out_dir}/{tag}.md"), &md).map_err(|e| e.to_string())?;
    let per_json: Vec<serde_json::Value> = per.iter().zip(&tq).map(|(x, q)| json!({"e": q.e, "rel": q.rel, "ans": q.ans, "rank_learned": x.0, "rank_raw": x.1, "rank_pop": x.2, "within2": x.3})).collect();
    std::fs::write(format!("{out_dir}/{tag}.json"), serde_json::to_string(&json!({"dataset": name, "seed": seed, "profile": p, "identity": lam, "k": k, "cap": cap, "shortlist": shortlist, "fac": fac_w, "feedback_top": fb_top, "feedback_events": n_fb, "entities": n, "rows": rows, "per_query": per_json})).unwrap()).map_err(|e| e.to_string())?;
    print!("{md}");
    Ok(())
}
