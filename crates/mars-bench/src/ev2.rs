//! EV2 (pre-registered, docs/PREREGISTRATION.md): MARS rankings for CodeNet
//! Python800 code retrieval, with the frozen v0.1 encoder, Mode K and mapper.
//!
//! Input: `tools/ev2_codenet.py prepare WORK` (vocab.mars, cases.mars,
//! manifest.json). For every program: the fingerprint-literal top-100 over all
//! other programs (Mode K, exhaustive), re-ranked by ½FAC + ½FP (E9's best
//! configuration, on a shortlist). Output: WORK/mars_rankings.json with both
//! rankings (case names, depth 100); metrics are computed by
//! `tools/ev2_codenet.py evaluate`.

use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_index::ModeK;
use mars_map::{MapConfig, Mapper};
use mars_rel::{CaseId, Kb};
use rayon::prelude::*;
use serde_json::json;
use std::time::Instant;

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "");
    let depth = args.usize("depth", 100);
    let fac_w = args.f64("fac", 0.5);
    let t0 = Instant::now();
    let mut kb = Kb::new();
    for f in ["vocab.mars", "cases.mars"] {
        kb.load_str(&std::fs::read_to_string(format!("{dir}/{f}")).map_err(|e| format!("{dir}/{f}: {e}"))?).map_err(|e| e.to_string())?;
    }
    let man: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/manifest.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let names: Vec<String> = man["cases"].as_array().ok_or("manifest: cases")?.iter().filter_map(|m| m["case"].as_str().map(String::from)).collect();
    let cases: Vec<CaseId> = names.iter().map(|n| kb.case_by_name(n).ok_or(format!("missing case {n}"))).collect::<Result<_, _>>()?;
    let n = cases.len();
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let raw: Vec<Features> = cases.par_iter().map(|&c| fx.extract(c)).collect();
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
    let scorer = index.scorer(&Profile::literal().weights);
    let mapper = Mapper::new(&kb, MapConfig::default());
    let self_s: Vec<f64> = cases.par_iter().map(|&c| mapper.score(c, c) as f64).collect();
    eprintln!("[ev2] {n} programs encoded ({:.1?})", t0.elapsed());
    let fac = |q: usize, c: usize| -> f64 {
        let raw = mapper.score(cases[c], cases[q]) as f64;
        if raw == 0.0 || self_s[c] == 0.0 || self_s[q] == 0.0 {
            0.0
        } else {
            (raw / (self_s[q] * self_s[c]).sqrt()).min(1.0)
        }
    };
    // E34 arm A2: FAC scores for externally supplied candidate lists (e.g. a code
    // embedding's top-100): WORK/<cands> = {case: [case, …]} -> WORK/fac_cands.json,
    // aligned with the lists; candidates not encoded in WORK score 0.
    let cands_file = args.str("cands", "");
    if !cands_file.is_empty() {
        let cj: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/{cands_file}")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let pos: std::collections::HashMap<&str, usize> = names.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
        let out: serde_json::Map<String, serde_json::Value> = names
            .par_iter()
            .enumerate()
            .map(|(q, nm)| {
                let list: Vec<f64> = cj[nm].as_array().map(|a| a.iter().map(|x| x.as_str().and_then(|s| pos.get(s)).map_or(0.0, |&c| fac(q, c))).collect()).unwrap_or_default();
                (nm.clone(), json!(list))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        std::fs::write(format!("{dir}/fac_cands.json"), serde_json::to_string(&out).unwrap()).map_err(|e| e.to_string())?;
        eprintln!("[ev2] FAC for supplied candidates done ({:.1?})", t0.elapsed());
        return Ok(());
    }
    let mut fp_rank: Vec<Vec<usize>> = Vec::with_capacity(n);
    let mut fused_rank: Vec<Vec<usize>> = Vec::with_capacity(n);
    let ids: Vec<usize> = (0..n).collect();
    for chunk in ids.chunks(512) {
        let qv: Vec<&[u64]> = chunk.iter().map(|&i| fps[i].as_slice()).collect();
        let hits = index.search_batch(&qv, &scorer, depth + 1);
        let res: Vec<(Vec<usize>, Vec<usize>)> = chunk
            .par_iter()
            .zip(hits.par_iter())
            .map(|(&q, h)| {
                let fp: Vec<(usize, f64)> = h.iter().filter(|x| x.id as usize != q).take(depth).map(|x| (x.id as usize, x.score as f64)).collect();
                let mut fu: Vec<(usize, f64)> = fp
                    .iter()
                    .map(|&(c, s)| (c, fac_w * fac(q, c) + (1.0 - fac_w) * s))
                    .collect();
                fu.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                (fp.iter().map(|x| x.0).collect(), fu.iter().map(|x| x.0).collect())
            })
            .collect();
        for (a, b) in res {
            fp_rank.push(a);
            fused_rank.push(b);
        }
        eprintln!("[ev2] {}/{n} queries ({:.1?})", fp_rank.len(), t0.elapsed());
    }
    eprintln!("[ev2] rankings done ({:.1?})", t0.elapsed());
    let to_names = |r: &Vec<Vec<usize>>| -> serde_json::Map<String, serde_json::Value> { names.iter().zip(r).map(|(n, l)| (n.clone(), json!(l.iter().map(|&i| names[i].as_str()).collect::<Vec<_>>()))).collect() };
    std::fs::write(format!("{dir}/mars_rankings.json"), serde_json::to_string(&json!({"fp": to_names(&fp_rank), "fused": to_names(&fused_rank), "depth": depth, "fac": fac_w, "profile": "literal"})).unwrap()).map_err(|e| e.to_string())?;
    Ok(())
}
