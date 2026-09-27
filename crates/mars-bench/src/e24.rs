//! E24: story-analogy retrieval at scale (MAC stage for an LLM verifier).
//!
//! All StoryAnalogy multiple-choice stories (sources, analogies, noun and
//! random distractors of every question) form one memory. Each source
//! queries the whole memory (minus itself); relevant = its analogy
//! (*target*). MARS rankings are written per query (top-50) so that
//! `tools/e24_pipeline.py` can compare them with text retrievers and run an
//! LLM verifier on each retriever's top-k.
//!
//! MARS methods: fingerprint analogy profile; analogy − 0.5·surface (C0);
//! and both re-ranked with FAC over the fingerprint top-64 (fused 0.3·FAC +
//! 0.7·score).

use crate::metrics::{mrr, recall_at};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_map::{MapConfig, Mapper};
use mars_rel::{CaseId, Kb};
use rayon::prelude::*;
use serde_json::json;
use std::fmt::Write as _;

const METHODS: [&str; 4] = ["fingerprint analogy", "analogy − 0.5·surface", "fused FAC + fingerprint analogy", "fused FAC + (analogy − 0.5·surface)"];

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/storyanalogy");
    let mc_path = args.str("mc", &format!("{dir}/storyanalogy_multiple_choice.json"));
    let out_dir = args.str("out", "results/E24");
    let top_out = args.usize("top", 50);
    let mut kb = Kb::new();
    for f in ["vocab.mars", "cases.mars"] {
        kb.load_str(&std::fs::read_to_string(format!("{dir}/{f}")).map_err(|e| format!("{dir}/{f}: {e}"))?).map_err(|e| format!("{f}: {e}"))?;
    }
    let mc: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(&mc_path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    // Memory: every story of every question.
    let mut names: Vec<String> = Vec::new();
    for (i, q) in mc.iter().enumerate() {
        names.push(format!("q{i}-s"));
        for j in 0..q["choices"].as_array().map(|a| a.len()).unwrap_or(0) {
            names.push(format!("q{i}-c{j}"));
        }
    }
    let cases: Vec<CaseId> = names.iter().map(|n| kb.case_by_name(n).ok_or(format!("missing case {n}"))).collect::<Result<_, _>>()?;
    let pos: rustc_hash::FxHashMap<&str, usize> = names.iter().enumerate().map(|(i, n)| (n.as_str(), i)).collect();
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let raw: Vec<Features> = cases.par_iter().map(|&c| fx.extract(c)).collect();
    let stats = FeatureStats::fit(raw.iter());
    let sk = Sketcher::new(Layout::default(), 0xF1);
    let fps: Vec<_> = raw
        .into_par_iter()
        .map(|mut f| {
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f)
        })
        .collect();
    let mapper = Mapper::new(&kb, MapConfig::default());
    let selfs: Vec<f64> = cases.par_iter().map(|&c| mapper.score(c, c) as f64).collect();
    let an = Profile::analogy();
    let queries: Vec<(usize, usize, usize)> = mc.iter().enumerate().map(|(i, q)| (i, pos[format!("q{i}-s").as_str()], pos[format!("q{i}-c{}", q["answer"].as_u64().unwrap()).as_str()])).collect();
    // Per query: per method, the ranked list (index, score).
    let res: Vec<Vec<Vec<(usize, f64)>>> = queries
        .par_iter()
        .map(|&(_, qi, _)| {
            let base: Vec<(usize, f64, f64)> = (0..cases.len())
                .filter(|&c| c != qi)
                .map(|c| {
                    let sims = sk.channel_sims(&fps[qi], &fps[c]);
                    let a = an.score(&sims);
                    (c, a, a - 0.5 * sims[0])
                })
                .collect();
            let rank = |v: &mut Vec<(usize, f64)>| v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let mut m0: Vec<(usize, f64)> = base.iter().map(|x| (x.0, x.1)).collect();
            let mut m1: Vec<(usize, f64)> = base.iter().map(|x| (x.0, x.2)).collect();
            rank(&mut m0);
            rank(&mut m1);
            let fuse = |v: &[(usize, f64)]| {
                let mut top: Vec<(usize, f64)> = v
                    .iter()
                    .take(64)
                    .map(|&(c, s)| {
                        let raw = mapper.score(cases[c], cases[qi]) as f64;
                        let fac = if raw == 0.0 { 0.0 } else { (raw / (selfs[qi] * selfs[c]).sqrt()).min(1.0) };
                        (c, 0.3 * fac + 0.7 * s)
                    })
                    .collect();
                top.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                top.extend(v.iter().skip(64).copied());
                top
            };
            let m2 = fuse(&m0);
            let m3 = fuse(&m1);
            vec![m0, m1, m2, m3]
        })
        .collect();
    let mut md = String::new();
    writeln!(md, "# E24: story-analogy retrieval over the pooled StoryAnalogy memory — MARS\n").unwrap();
    writeln!(md, "Memory: {} stories (all sources, analogies and distractors of {} questions). Each source retrieves among the other {}; relevant = its analogy.\n", cases.len(), mc.len(), cases.len() - 1).unwrap();
    writeln!(md, "| method | R@1 | R@5 | R@10 | R@50 | MRR |\n|---|---|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    for (m, name) in METHODS.iter().enumerate() {
        let ranks: Vec<usize> = queries.iter().zip(&res).map(|(&(_, _, t), r)| r[m].iter().position(|x| x.0 == t).unwrap_or(usize::MAX / 2)).collect();
        writeln!(md, "| MARS {name} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |", recall_at(&ranks, 1), recall_at(&ranks, 5), recall_at(&ranks, 10), recall_at(&ranks, 50), mrr(&ranks)).unwrap();
        rows.push(json!({"method": format!("MARS {name}"), "r1": recall_at(&ranks, 1), "r5": recall_at(&ranks, 5), "r10": recall_at(&ranks, 10), "r50": recall_at(&ranks, 50), "mrr": mrr(&ranks)}));
    }
    let per_query: Vec<serde_json::Value> = queries
        .iter()
        .zip(&res)
        .map(|(&(i, _, _), r)| json!({"q": i, "rankings": METHODS.iter().enumerate().map(|(m, name)| (format!("MARS {name}"), json!(r[m].iter().take(top_out).map(|x| &names[x.0]).collect::<Vec<_>>()))).collect::<serde_json::Map<String, serde_json::Value>>()}))
        .collect();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E24-mars.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E24-mars.json"), serde_json::to_string_pretty(&json!({"memory": cases.len(), "rows": rows, "per_query": per_query})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
