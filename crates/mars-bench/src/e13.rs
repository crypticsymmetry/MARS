//! E13: analogical bootstrapping of vocabulary alignment.
//!
//! Start from the E0 *unresolved* setting: every domain uses its own names
//! for first-order relations (`bio:inhibits`, `law:inhibits`, ...) with no
//! taxonomy link, so structure can only be compared through the canonical
//! higher-order relations and anonymous shapes (E0: TA-top ≈ 0.55).
//! Each round:
//!   1. retrieve cross-domain neighbours with the current fingerprints;
//!   2. map each pair with *wildcard* matching (different relations of the
//!      same arity may correspond at a low local score, anchored by shared
//!      higher-order structure);
//!   3. accumulate evidence for predicate correspondences from strong mappings;
//!   4. align mutual-best predicate pairs under a new canonical predicate;
//!   5. re-encode and re-evaluate.
//! No labels are used; ground truth (same canonical base name) only scores
//! the alignments.

use crate::metrics::{mean, win_rate};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Layout, Profile, Sketcher, N_CHANNELS};
use mars_gen::{generate, Dataset, GenConfig, Naming, VariantClass};
use mars_index::ModeK;
use mars_map::{MapConfig, Mapper};
use mars_rel::{PredKind, Sym, Term};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

fn domain_of(name: &str) -> Option<&str> {
    name.split_once(':').map(|x| x.0)
}

fn base_of(name: &str) -> &str {
    name.split_once(':').map(|x| x.1).unwrap_or(name)
}

fn fingerprints(ds: &Dataset, seed: u64) -> (Vec<Vec<u64>>, Layout) {
    let fx = FeatureExtractor::new(&ds.kb, FeatureConfig::default());
    let mut feats: Vec<_> = ds.items.par_iter().map(|it| fx.extract(it.case)).collect();
    let stats = FeatureStats::fit(feats.iter());
    feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
    let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);
    (feats.par_iter().map(|f| sk.sketch(f).into_words()).collect(), sk.layout.clone())
}

/// TA-top for fingerprint, FAC and fused scores (per group, as in E0/E2).
fn evaluate(ds: &Dataset, fps: &[Vec<u64>], layout: &Layout) -> (f64, f64, f64) {
    let profile = Profile::analogy();
    let mapper = Mapper::new(&ds.kb, MapConfig::default());
    let res: Vec<[(f64, f64); 3]> = ds
        .groups
        .par_iter()
        .map(|g| {
            let b = g.base;
            let sb = mapper.score(ds.case(b), ds.case(b)) as f64;
            let score = |i: usize| -> (f64, f64) {
                let fp = profile.score(&mars_encode::channel_sims(layout, &fps[b], &fps[i]));
                let s = mapper.score(ds.case(i), ds.case(b)) as f64;
                let si = mapper.score(ds.case(i), ds.case(i)) as f64;
                let fac = if s == 0.0 { 0.0 } else { (s / (sb * si).sqrt()).min(1.0) };
                (fp, fac)
            };
            let ta = score(g.ta);
            let others: Vec<(f64, f64)> = g.members().into_iter().filter(|(c, _)| *c != VariantClass::TA && *c != VariantClass::LS).map(|(_, i)| score(i)).collect();
            let best = |f: &dyn Fn(&(f64, f64)) -> f64| others.iter().map(f).fold(f64::MIN, f64::max);
            let fused = |x: &(f64, f64)| 0.3 * x.1 + 0.7 * x.0;
            [(ta.0, best(&|x| x.0)), (ta.1, best(&|x| x.1)), (fused(&ta), best(&fused))]
        })
        .collect();
    let col = |k: usize| win_rate(&res.iter().map(|r| r[k]).collect::<Vec<_>>());
    (col(0), col(1), col(2))
}

pub fn run(args: &Args) -> Result<(), String> {
    let groups = args.usize("groups", 1000);
    let seed = args.u64("seed", 1);
    let rounds = args.usize("rounds", 3);
    let neighbours = args.usize("neighbours", 5);
    let theta = args.f64("theta", 0.4);
    let wildcard = args.f64("wildcard", 0.3) as f32;
    let min_evidence = args.f64("min-evidence", 1.0);
    let out_dir = args.str("out", "results/E13");
    let reestimate = args.str("reestimate", "yes") == "yes";
    let t0 = Instant::now();
    let gcfg = GenConfig { seed, n_groups: groups, naming: Naming::Unresolved, distractors: args.usize("distractors", 2), ..Default::default() };
    let mut ds = generate(&gcfg);
    let n = ds.items.len();

    let mut md = String::new();
    writeln!(md, "# E13: analogical bootstrapping of vocabulary alignment\n").unwrap();
    writeln!(md, "E0 *unresolved* setting ({groups} groups, {n} cases, 2 distractors): each of 12 domains names its first-order relations differently (`bio:inhibits`, `law:inhibits`, …) with no taxonomy. Each round retrieves {neighbours} cross-domain neighbours per case, maps pairs with wildcard matching (local score {wildcard}), harvests predicate correspondences from mappings with normalized score ≥ {theta}, and aligns mutual-best pairs (evidence ≥ {min_evidence}) under new canonical predicates. No labels are used; ground truth only scores the alignment.\n").unwrap();
    writeln!(md, "| round | aligned predicates | clusters | alignment precision (pairs) | coverage | TA-top fingerprint | TA-top FAC | TA-top fused | time |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|---|").unwrap();

    // All unresolved first-order predicates actually used.
    let preds: Vec<Sym> = {
        let mut v: Vec<Sym> = Vec::new();
        for it in &ds.items {
            for e in ds.kb.case_exprs(it.case) {
                let f = ds.kb.expr(e).functor;
                if domain_of(ds.kb.name(f)).is_some() && !v.contains(&f) {
                    v.push(f);
                }
            }
        }
        v
    };
    let mut parent: FxHashMap<Sym, Sym> = FxHashMap::default(); // predicate -> cluster representative
    let find = |parent: &FxHashMap<Sym, Sym>, mut x: Sym| -> Sym {
        while let Some(&p) = parent.get(&x) {
            if p == x {
                break;
            }
            x = p;
        }
        x
    };
    let mut rows = Vec::new();
    let mut mb_precisions: Vec<f64> = Vec::new();
    for round in 0..=rounds {
        let tr = Instant::now();
        let (fps, layout) = fingerprints(&ds, seed);
        let (fp_top, fac_top, fused_top) = evaluate(&ds, &fps, &layout);
        // Alignment quality so far.
        let aligned: Vec<Sym> = preds.iter().copied().filter(|&p| find(&parent, p) != p || parent.values().any(|&q| q == p && find(&parent, p) == p && parent.iter().any(|(k, _)| *k != p && find(&parent, *k) == p))).collect();
        let mut clusters: FxHashMap<Sym, Vec<Sym>> = FxHashMap::default();
        for &p in &preds {
            clusters.entry(find(&parent, p)).or_default().push(p);
        }
        let multi: Vec<&Vec<Sym>> = clusters.values().filter(|c| c.len() > 1).collect();
        let (mut good, mut pairs) = (0usize, 0usize);
        for c in &multi {
            for i in 0..c.len() {
                for j in i + 1..c.len() {
                    pairs += 1;
                    good += (base_of(ds.kb.name(c[i])) == base_of(ds.kb.name(c[j]))) as usize;
                }
            }
        }
        let n_aligned: usize = multi.iter().map(|c| c.len()).sum();
        let precision = if pairs == 0 { f64::NAN } else { good as f64 / pairs as f64 };
        let coverage = n_aligned as f64 / preds.len() as f64;
        let _ = aligned;
        if round == rounds {
            writeln!(md, "| {round} (final) | {n_aligned} / {} | {} | {precision:.3} | {coverage:.3} | {fp_top:.3} | {fac_top:.3} | {fused_top:.3} | {:.1?} |", preds.len(), multi.len(), tr.elapsed()).unwrap();
            rows.push(json!({"round": round, "aligned": n_aligned, "clusters": multi.len(), "precision": precision, "coverage": coverage, "fp": fp_top, "fac": fac_top, "fused": fused_top}));
            break;
        }

        // 1. Cross-domain neighbours.
        let mut idx = ModeK::with_capacity(layout.clone(), n);
        for f in &fps {
            idx.push(f);
        }
        let sc = idx.scorer(&Profile::analogy().weights);
        let pairs_to_map: Vec<(usize, usize)> = (0..n)
            .into_par_iter()
            .flat_map_iter(|i| {
                let d = ds.items[i].domain;
                idx.search_serial(&fps[i], &sc, neighbours * 8 + 1)
                    .into_iter()
                    .filter(|h| h.id as usize != i && ds.items[h.id as usize].domain != d)
                    .take(neighbours)
                    .map(|h| (i, h.id as usize))
                    .collect::<Vec<_>>()
            })
            .collect();
        // 2–3. Wildcard mappings -> predicate correspondence evidence.
        let wmapper = Mapper::new(&ds.kb, MapConfig { wildcard: Some(wildcard), ..Default::default() });
        let self_scores: Vec<f64> = (0..n).into_par_iter().map(|i| wmapper.score(ds.case(i), ds.case(i)) as f64).collect();
        let evidence: FxHashMap<(Sym, Sym), f64> = pairs_to_map
            .par_iter()
            .fold(FxHashMap::default, |mut acc: FxHashMap<(Sym, Sym), f64>, &(i, j)| {
                if let Some(m) = wmapper.best(ds.case(j), ds.case(i)) {
                    let norm = m.score as f64 / (self_scores[i] * self_scores[j]).sqrt();
                    if norm >= theta {
                        for (b, t) in &m.correspondences {
                            if let (Term::Expr(be), Term::Expr(te)) = (b, t) {
                                let (fb, ft) = (ds.kb.expr(*be).functor, ds.kb.expr(*te).functor);
                                if fb != ft && ds.kb.vocab.kind(fb) == PredKind::Relation && domain_of(ds.kb.name(fb)).is_some() && domain_of(ds.kb.name(ft)).is_some() {
                                    let key = if fb < ft { (fb, ft) } else { (ft, fb) };
                                    *acc.entry(key).or_insert(0.0) += norm;
                                }
                            }
                        }
                    }
                }
                acc
            })
            .reduce(FxHashMap::default, |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0.0) += v;
                }
                a
            });
        // 4. Mutual best partner per (predicate, other domain).
        let mut best: FxHashMap<(Sym, String), (Sym, f64)> = FxHashMap::default();
        for (&(a, b), &w) in &evidence {
            for (x, y) in [(a, b), (b, a)] {
                let dom = domain_of(ds.kb.name(y)).unwrap().to_string();
                let e = best.entry((x, dom)).or_insert((y, 0.0));
                if w > e.1 {
                    *e = (y, w);
                }
            }
        }
        let mut merges: Vec<(Sym, Sym, f64)> = Vec::new();
        for (&(a, b), &w) in &evidence {
            if w < min_evidence {
                continue;
            }
            let (da, db) = (domain_of(ds.kb.name(a)).unwrap().to_string(), domain_of(ds.kb.name(b)).unwrap().to_string());
            if best.get(&(a, db)).map(|x| x.0) == Some(b) && best.get(&(b, da)).map(|x| x.0) == Some(a) {
                merges.push((a, b, w));
            }
        }
        merges.sort_by(|x, y| y.2.total_cmp(&x.2));
        let mb_prec = mean(&merges.iter().map(|(a, b, _)| (base_of(ds.kb.name(*a)) == base_of(ds.kb.name(*b))) as u8 as f64).collect::<Vec<_>>());
        eprintln!("[e13] round {round}: {} mutual-best pairs, pair precision {mb_prec:.3}", merges.len());
        mb_precisions.push(mb_prec);
        // EM-style re-estimation: rebuild the alignment from this round's evidence
        // (retrieval already benefits from the previous alignment), so early
        // mistakes are not entrenched.
        if reestimate {
            parent.clear();
            for &p in &preds {
                ds.kb.vocab.set_parents(p, Vec::new());
            }
        }
        // Greedy union by evidence, never putting two predicates of one domain in a cluster.
        let mut doms: FxHashMap<Sym, Vec<String>> = FxHashMap::default();
        for &p in &preds {
            let r = find(&parent, p);
            doms.entry(r).or_default().push(domain_of(ds.kb.name(p)).unwrap().to_string());
        }
        for (a, b, _) in merges {
            let (ra, rb) = (find(&parent, a), find(&parent, b));
            if ra == rb {
                continue;
            }
            let (da, db) = (doms.get(&ra).cloned().unwrap_or_default(), doms.get(&rb).cloned().unwrap_or_default());
            if da.iter().any(|d| db.contains(d)) {
                continue;
            }
            parent.insert(rb, ra);
            parent.entry(ra).or_insert(ra);
            doms.entry(ra).or_default().extend(db);
            doms.remove(&rb);
        }
        // 5. Materialize clusters as canonical predicates in the vocabulary.
        let mut reps: FxHashMap<Sym, Vec<Sym>> = FxHashMap::default();
        for &p in &preds {
            reps.entry(find(&parent, p)).or_default().push(p);
        }
        for (rep, members) in reps {
            if members.len() < 2 {
                continue;
            }
            let cname = format!("aligned-{}", ds.kb.name(rep).replace(':', "-"));
            let c = ds.kb.declare(&cname, Some(2), PredKind::Relation, false, &[]);
            for m in members {
                ds.kb.vocab.set_parents(m, vec![c]);
            }
        }
        writeln!(md, "| {round} | {n_aligned} / {} | {} | {precision:.3} | {coverage:.3} | {fp_top:.3} | {fac_top:.3} | {fused_top:.3} | {:.1?} |", preds.len(), multi.len(), tr.elapsed()).unwrap();
        rows.push(json!({"round": round, "aligned": n_aligned, "clusters": multi.len(), "precision": precision, "coverage": coverage, "fp": fp_top, "fac": fac_top, "fused": fused_top, "pairs_mapped": pairs_to_map.len(), "evidence_pairs": evidence.len()}));
        eprintln!("[e13] round {round}: {} pairs mapped, {} evidence pairs ({:.1?})", pairs_to_map.len(), evidence.len(), t0.elapsed());
    }
    // Reference: canonical naming (the oracle vocabulary).
    let oracle = generate(&GenConfig { naming: Naming::Canonical, ..gcfg.clone() });
    let (ofps, olayout) = fingerprints(&oracle, seed);
    let (ofp, ofac, ofused) = evaluate(&oracle, &ofps, &olayout);
    writeln!(md, "| oracle (canonical names) | — | — | 1.000 | 1.000 | {ofp:.3} | {ofac:.3} | {ofused:.3} | |").unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E13.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E13.json"), serde_json::to_string_pretty(&json!({"rows": rows, "oracle": [ofp, ofac, ofused]})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    let _ = mean(&[]);
    Ok(())
}
