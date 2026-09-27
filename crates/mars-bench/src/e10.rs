//! E10: why do partial analogues degrade with corpus size? Scoring-function
//! study on the E3 setting (Mode K top-64 candidates, re-ranked).
//!
//! Variants: mapper local scores plain vs IDF-weighted (informativeness);
//! normalization of the structural score S(c→q) by the self-scores:
//! geometric S/√(Sqq·Scc), Dice 2S/(Sqq+Scc), query coverage S/Sqq,
//! candidate coverage S/Scc; fusion weight w on the structural part.
//! Also attributes the default method's failures to the kind of winner.

use crate::metrics::mean;
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_gen::{generate, GenConfig, Naming, VariantClass};
use mars_index::{Hit, ModeK};
use mars_map::{predicate_idf, MapConfig, Mapper};
use mars_rel::CaseId;
use rayon::prelude::*;
use serde_json::json;
use std::fmt::Write as _;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

const NORMS: [&str; 4] = ["geometric", "dice", "query-coverage", "candidate-coverage"];
const WS: [f64; 5] = [0.0, 0.3, 0.5, 0.7, 1.0];

fn normalize(kind: usize, s: f64, sq: f64, sc: f64) -> f64 {
    if s <= 0.0 || sq <= 0.0 || sc <= 0.0 {
        return 0.0;
    }
    (match kind {
        0 => s / (sq * sc).sqrt(),
        1 => 2.0 * s / (sq + sc),
        2 => s / sq,
        _ => s / sc,
    })
    .min(1.0)
}

struct Cand {
    item: usize,
    fp: f64,
    s: [f64; 2],
}

pub fn run(args: &Args) -> Result<(), String> {
    let groups = args.usize("groups", 12500);
    let n_queries = args.usize("queries", 1000).min(groups);
    let k = args.usize("k", 64);
    let seed = args.u64("seed", 1);
    let (ops, severity) = args.perturbation();
    let out_dir = args.str("out", "results/E10");
    let tag = args.str("tag", &format!("n{}-s{severity}-c{}", groups * 8, args.usize("compose", 1)));
    let t0 = Instant::now();
    let gcfg = GenConfig { seed, n_groups: groups, naming: Naming::Canonical, distractors: 2, perturb_ops: ops, severity, compose: args.usize("compose", 1), ..Default::default() };
    let ds = generate(&gcfg);
    let n = ds.items.len();
    let fx = FeatureExtractor::new(&ds.kb, FeatureConfig::default());
    let mut feats: Vec<Features> = ds.items.par_iter().map(|it| fx.extract(it.case)).collect();
    let stats = FeatureStats::fit(feats.iter());
    feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
    let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);
    let fps: Vec<Vec<u64>> = feats.par_iter().map(|f| sk.sketch(f).into_words()).collect();
    drop(feats);
    let mut idx = ModeK::with_capacity(sk.layout.clone(), n);
    for f in &fps {
        idx.push(f);
    }
    let scorer = idx.scorer(&Profile::analogy().weights);
    eprintln!("[e10] {n} cases indexed ({:.1?})", t0.elapsed());

    let all_cases: Vec<CaseId> = ds.items.iter().map(|it| it.case).collect();
    let idf = Arc::new(predicate_idf(&ds.kb, &all_cases));
    let mappers = [Mapper::new(&ds.kb, MapConfig::default()), Mapper::new(&ds.kb, MapConfig { pred_weights: Some(idf), ..Default::default() })];
    let selfs: [Vec<OnceLock<f64>>; 2] = [(0..n).map(|_| OnceLock::new()).collect(), (0..n).map(|_| OnceLock::new()).collect()];
    let self_score = |m: usize, i: usize| *selfs[m][i].get_or_init(|| mappers[m].score(ds.case(i), ds.case(i)) as f64);

    // Candidates per query.
    let qs: Vec<&[u64]> = (0..n_queries).map(|qi| fps[ds.groups[qi].base].as_slice()).collect();
    let lists: Vec<Vec<Hit>> = qs.chunks(64).flat_map(|c| idx.search_batch(c, &scorer, k + 2)).collect();
    let per_q: Vec<Vec<Cand>> = lists
        .par_iter()
        .enumerate()
        .map(|(qi, l)| {
            let g = &ds.groups[qi];
            let q = g.base;
            l.iter()
                .filter(|h| h.id as usize != g.base && h.id as usize != g.ls)
                .take(k)
                .map(|h| {
                    let c = h.id as usize;
                    let s = [0, 1].map(|m| mappers[m].score(ds.case(c), ds.case(q)) as f64);
                    let _ = (self_score(0, c), self_score(1, c), self_score(0, q), self_score(1, q));
                    Cand { item: c, fp: idx.score(&scorer, &fps[q], c as u32) as f64, s }
                })
                .collect()
        })
        .collect();
    eprintln!("[e10] candidates scored ({:.1?})", t0.elapsed());

    let disc: Vec<bool> = (0..n_queries).map(|qi| ds.groups[qi].discriminable(&ds)).collect();
    let mac_recall = mean(&(0..n_queries).map(|qi| per_q[qi].iter().any(|c| c.item == ds.groups[qi].ta) as u8 as f64).collect::<Vec<_>>());
    let winner = |qi: usize, m: usize, norm: usize, w: f64| -> usize {
        let q = ds.groups[qi].base;
        per_q[qi]
            .iter()
            .map(|c| (c.item, w * normalize(norm, c.s[m], self_score(m, q), self_score(m, c.item)) + (1.0 - w) * c.fp))
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|x| x.0)
            .unwrap_or(usize::MAX)
    };

    let mut md = String::new();
    writeln!(md, "# E10: scoring-function study ({tag})\n").unwrap();
    writeln!(md, "Corpus {n} cases, {n_queries} queries (group bases; target TA; base and LS excluded), candidates = Mode K top-{k} (TA among them for {:.3} of queries: the ceiling). Perturbation {:?} × {severity}. Accuracy = TA ranked first after re-ranking by `w·norm(S) + (1−w)·fp`. Discriminable queries: {}/{n_queries}.\n",
        mac_recall, gcfg.perturb_ops.iter().map(|o| o.name()).collect::<Vec<_>>(), disc.iter().filter(|&&d| d).count()).unwrap();
    writeln!(md, "| mapper | normalization | w=0 (fp only) | w=0.3 | w=0.5 | w=0.7 | w=1 (FAC only) | best w: discriminable acc |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    for m in 0..2 {
        for (ni, nname) in NORMS.iter().enumerate() {
            let accs: Vec<f64> = WS.iter().map(|&w| mean(&(0..n_queries).into_par_iter().map(|qi| (winner(qi, m, ni, w) == ds.groups[qi].ta) as u8 as f64).collect::<Vec<_>>())).collect();
            let (bi, _) = accs.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap();
            let dacc = mean(&(0..n_queries).filter(|&qi| disc[qi]).map(|qi| (winner(qi, m, ni, WS[bi]) == ds.groups[qi].ta) as u8 as f64).collect::<Vec<_>>());
            let mname = if m == 0 { "plain" } else { "IDF-weighted" };
            writeln!(md, "| {mname} | {nname} | {} | {dacc:.3} (w={}) |", accs.iter().map(|a| format!("{a:.3}")).collect::<Vec<_>>().join(" | "), WS[bi]).unwrap();
            rows.push(json!({"mapper": mname, "norm": nname, "acc_by_w": accs, "best_w": WS[bi], "disc_acc_best_w": dacc}));
        }
    }

    // Failure attribution for the default scoring (plain, geometric, w = 0.5).
    let mut causes: std::collections::BTreeMap<String, usize> = Default::default();
    let mut size_ratio = Vec::new();
    for qi in 0..n_queries {
        let wn = winner(qi, 0, 0, 0.5);
        let g = &ds.groups[qi];
        if wn == g.ta {
            continue;
        }
        let key = if wn == usize::MAX {
            "no candidate".to_string()
        } else {
            let it = ds.item(wn);
            let own = it.group == g.base_group(&ds);
            let ta_in = per_q[qi].iter().any(|c| c.item == g.ta);
            format!("{} {}{}", if own { "own" } else { "other-group" }, it.class.name(), if ta_in { "" } else { " (TA not in top-k)" })
        };
        *causes.entry(key).or_insert(0) += 1;
        if wn != usize::MAX {
            let sz = |i: usize| ds.kb.case_exprs(ds.case(i)).len() as f64;
            size_ratio.push(sz(wn) / sz(g.ta));
        }
    }
    writeln!(md, "\n## Failures of the default (plain, geometric, w = 0.5): what wins instead of TA\n").unwrap();
    writeln!(md, "| winner | count |\n|---|---|").unwrap();
    for (k2, v) in &causes {
        writeln!(md, "| {k2} | {v} |").unwrap();
    }
    writeln!(md, "\nMean size ratio winner/TA (expressions): {:.2}.", mean(&size_ratio)).unwrap();

    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E10-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E10-{tag}.json"), serde_json::to_string_pretty(&json!({"tag": tag, "mac_recall": mac_recall, "rows": rows, "failures": causes})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    let _ = VariantClass::TA;
    Ok(())
}
