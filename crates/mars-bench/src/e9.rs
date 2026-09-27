//! E9: program analogy on real code (docs/EXPERIMENTS.md §4, E9).
//!
//! Corpus: every function of three independently written Python algorithm
//! packages, converted automatically to relational cases by
//! `tools/py2mars.py` (run `tools/fetch_e9_corpus.sh` first).
//!
//! * Task A: the same algorithm by a different author. Query = the main
//!   function of `<pkg>/<category>/<stem>.py`; relevant = main functions
//!   with the same file stem in *other* packages; ranked among all functions.
//! * Task B: same-category retrieval among main functions (e.g. a sort
//!   should retrieve other sorts).

use crate::metrics::{mean, mrr, recall_at};
use crate::Args;
use mars_encode::{channel_cosines, cosine, lexical_tokens, mac_content_vector, FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, SparseVec, N_CHANNELS};
use mars_map::{MapConfig, Mapper};
use mars_rel::{CaseId, Kb};
use rayon::prelude::*;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

pub(crate) struct Meta {
    pub case: CaseId,
    pub pkg: String,
    pub lang: String,
    pub norm: String,
    pub category: String,
    pub stem: String,
    pub is_main: bool,
}

/// Load `{dir}/vocab.mars`, every other `*.mars` file and the manifest.
pub(crate) fn load_corpus(dir: &str) -> Result<(Kb, Vec<Meta>), String> {
    let mut kb = Kb::new();
    kb.load_str(&std::fs::read_to_string(format!("{dir}/vocab.mars")).map_err(|e| format!("{e} (run tools/fetch_e9_corpus.sh)"))?).map_err(|e| e.to_string())?;
    let mut files: Vec<_> = std::fs::read_dir(dir).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map(|x| x == "mars").unwrap_or(false) && !p.ends_with("vocab.mars")).collect();
    files.sort();
    for f in &files {
        kb.load_str(&std::fs::read_to_string(f).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", f.display()))?;
    }
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/manifest.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let metas: Vec<Meta> = manifest
        .as_array()
        .ok_or("manifest must be a list")?
        .iter()
        .filter_map(|m| {
            let case = kb.case_by_name(m["case"].as_str()?)?;
            let stem: String = m["stem"].as_str()?.into();
            Some(Meta {
                case,
                pkg: m["pkg"].as_str()?.into(),
                lang: m["lang"].as_str().unwrap_or("py").into(),
                norm: normalize_stem(&stem),
                category: m["category"].as_str()?.into(),
                stem,
                is_main: m["is_main"].as_bool()?,
            })
        })
        .collect();
    Ok((kb, metas))
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/e9");
    let out_dir = args.str("out", if args.str("task", "cross-pkg") == "cross-lang" { "results/E12" } else { "results/E9" });
    let t0 = Instant::now();
    let cross_lang = args.str("task", "cross-pkg") == "cross-lang";
    let (kb, metas) = load_corpus(&dir)?;
    // Cross-package (E9) is defined on the Python corpus only; cross-language (E12) uses everything.
    let metas: Vec<Meta> = if cross_lang { metas } else { metas.into_iter().filter(|m| m.lang == "py").collect() };
    let n = metas.len();
    eprintln!("[e9] loaded {n} functions ({:.1?})", t0.elapsed());

    // ---------------- Representations
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let raw: Vec<Features> = metas.par_iter().map(|m| fx.extract(m.case)).collect();
    let stats = FeatureStats::fit(raw.iter());
    let feats: Vec<Features> = raw
        .into_iter()
        .map(|mut f| {
            stats.apply(&mut f, &[true; N_CHANNELS]);
            f
        })
        .collect();
    let sk = Sketcher::new(Layout::default(), 0xF1);
    let fps: Vec<_> = feats.par_iter().map(|f| sk.sketch(f)).collect();
    let lex_raw: Vec<SparseVec> = metas.iter().map(|m| lexical_tokens(&kb, m.case)).collect();
    let lex_stats = FeatureStats::fit(lex_raw.iter().map(|v| Features { channels: [v.clone(), vec![], vec![], vec![], vec![]] }).collect::<Vec<_>>().iter());
    let lex: Vec<SparseVec> = lex_raw.into_iter().map(|mut v| { lex_stats.apply_sparse(0, &mut v); v }).collect();
    let mac: Vec<SparseVec> = metas.iter().map(|m| mac_content_vector(&kb, m.case)).collect();
    let mapper = Mapper::new(&kb, MapConfig::default());
    let self_s: Vec<f64> = metas.par_iter().map(|m| mapper.score(m.case, m.case) as f64).collect();
    let analogy = Profile::analogy();
    let literal = Profile::literal();

    type Scorer<'a> = Box<dyn Fn(usize, usize) -> f64 + Sync + 'a>;
    let fac = |a: usize, b: usize| -> f64 {
        let s = mapper.score(metas[b].case, metas[a].case) as f64;
        if s == 0.0 { 0.0 } else { (s / (self_s[a] * self_s[b]).sqrt()).min(1.0) }
    };
    let fp = |a: usize, b: usize| analogy.score(&sk.channel_sims(&fps[a], &fps[b]));
    let methods: Vec<(&str, Scorer)> = vec![
        ("B2 lexical TF-IDF (identifiers + operations)", Box::new(|a, b| cosine(&lex[a], &lex[b]))),
        ("B4 MAC content vectors", Box::new(|a, b| cosine(&mac[a], &mac[b]))),
        ("exact analogy profile (+IDF)", Box::new(|a, b| analogy.score(&channel_cosines(&feats[a], &feats[b])))),
        ("fingerprint literal profile", Box::new(|a, b| literal.score(&sk.channel_sims(&fps[a], &fps[b])))),
        ("fingerprint analogy profile", Box::new(fp)),
        ("FAC only (exhaustive)", Box::new(fac)),
        ("fused ½FAC+½FP (exhaustive)", Box::new(|a, b| 0.5 * fac(a, b) + 0.5 * fp(a, b))),
        ("fused ½FAC+½FP-literal (exhaustive)", Box::new(|a, b| 0.5 * fac(a, b) + 0.5 * literal.score(&sk.channel_sims(&fps[a], &fps[b])))),
        ("fused 0.3FAC+0.7FP-literal (exhaustive)", Box::new(|a, b| 0.3 * fac(a, b) + 0.7 * literal.score(&sk.channel_sims(&fps[a], &fps[b])))),
    ];

    // Rank of each relevant item for query q under a scorer, over candidate pool.
    let rank_all = |score: &Scorer, q: usize, pool: &[usize], relevant: &dyn Fn(usize) -> bool| -> (Vec<usize>, Vec<usize>) {
        let mut s: Vec<(usize, f64)> = pool.iter().filter(|&&c| c != q).map(|&c| (c, score(q, c))).collect();
        s.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let order: Vec<usize> = s.iter().map(|x| x.0).collect();
        let ranks = order.iter().enumerate().filter(|(_, &c)| relevant(c)).map(|(r, _)| r).collect();
        (ranks, order)
    };

    let all: Vec<usize> = (0..n).collect();
    let mains: Vec<usize> = (0..n).filter(|&i| metas[i].is_main).collect();
    // Task A queries.
    let metas_ref = &metas;
    let counterpart = move |q: usize, c: usize| -> bool {
        let (a, b) = (&metas_ref[q], &metas_ref[c]);
        b.is_main && b.norm == a.norm && if cross_lang { b.lang != a.lang } else { b.pkg != a.pkg }
    };
    let queries_a: Vec<usize> = mains.iter().copied().filter(|&i| mains.iter().any(|&j| j != i && counterpart(i, j))).collect();
    let rel_a = |q: usize| move |c: usize| counterpart(q, c);
    // Task B queries: mains in categories with ≥ 3 mains.
    let cat_count = |c: &str| mains.iter().filter(|&&i| metas[i].category == c).count();
    let queries_b: Vec<usize> = mains.iter().copied().filter(|&i| cat_count(&metas[i].category) >= 3).collect();

    let mut md = String::new();
    writeln!(md, "# E9: program analogy on real code\n").unwrap();
    let pkgs: std::collections::BTreeSet<&str> = metas.iter().map(|m| m.pkg.as_str()).collect();
    writeln!(md, "Corpus: {n} functions from packages {:?}, converted to relational cases automatically (`tools/py2mars.py`, `tools/js2mars.py`; no per-program tuning). {} main functions. Runtime {:.1?}.\n", pkgs, mains.len(), t0.elapsed()).unwrap();
    writeln!(md, "## Task A: same algorithm, different {} ({} queries; ranked among all {n} functions)\n", if cross_lang { "language" } else { "author" }, queries_a.len()).unwrap();
    writeln!(md, "| method | R@1 | R@5 | R@10 | MRR |").unwrap();
    writeln!(md, "|---|---|---|---|---|").unwrap();
    let mut ja = Vec::new();
    let mut per_query: Vec<Vec<usize>> = vec![Vec::new(); queries_a.len()];
    for (name, sc) in &methods {
        let ranks: Vec<usize> = queries_a
            .par_iter()
            .map(|&q| {
                let rel = rel_a(q);
                rank_all(sc, q, &all, &rel).0.into_iter().min().unwrap_or(usize::MAX / 2)
            })
            .collect();
        for (i, r) in ranks.iter().enumerate() {
            per_query[i].push(*r);
        }
        writeln!(md, "| {name} | {:.3} | {:.3} | {:.3} | {:.3} |", recall_at(&ranks, 1), recall_at(&ranks, 5), recall_at(&ranks, 10), mrr(&ranks)).unwrap();
        ja.push(json!({"method": name, "r1": recall_at(&ranks, 1), "r5": recall_at(&ranks, 5), "r10": recall_at(&ranks, 10), "mrr": mrr(&ranks)}));
    }
    writeln!(md, "\nPer-query rank of the first correct counterpart (0 = top):\n").unwrap();
    write!(md, "| query |").unwrap();
    for (name, _) in &methods {
        write!(md, " {} |", name.split(' ').take(2).collect::<Vec<_>>().join(" ")).unwrap();
    }
    writeln!(md, "\n|---|{}", "---|".repeat(methods.len())).unwrap();
    for (i, &q) in queries_a.iter().enumerate() {
        write!(md, "| {}/{}/{} |", metas[q].pkg, metas[q].category, metas[q].stem).unwrap();
        for r in &per_query[i] {
            write!(md, " {} |", if *r >= usize::MAX / 4 { "–".to_string() } else { r.to_string() }).unwrap();
        }
        writeln!(md).unwrap();
    }

    writeln!(md, "\n## Task B: same-category retrieval among main functions ({} queries)\n", queries_b.len()).unwrap();
    writeln!(md, "| method | P@1 | P@5 | MRR (first same-category) |").unwrap();
    writeln!(md, "|---|---|---|---|").unwrap();
    let mut jb = Vec::new();
    for (name, sc) in &methods {
        let res: Vec<(f64, f64, usize)> = queries_b
            .par_iter()
            .map(|&q| {
                let rel = |c: usize| metas[c].category == metas[q].category;
                let (ranks, order) = rank_all(sc, q, &mains, &rel);
                let p1 = order.first().map(|&c| rel(c) as u8 as f64).unwrap_or(0.0);
                let p5 = order.iter().take(5).filter(|&&c| rel(c)).count() as f64 / 5.0;
                (p1, p5, ranks.into_iter().min().unwrap_or(usize::MAX / 2))
            })
            .collect();
        let (p1, p5): (Vec<f64>, Vec<f64>) = res.iter().map(|x| (x.0, x.1)).unzip();
        let ranks: Vec<usize> = res.iter().map(|x| x.2).collect();
        writeln!(md, "| {name} | {:.3} | {:.3} | {:.3} |", mean(&p1), mean(&p5), mrr(&ranks)).unwrap();
        jb.push(json!({"method": name, "p1": mean(&p1), "p5": mean(&p5), "mrr": mrr(&ranks)}));
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let stem_out = if cross_lang { "E12" } else { "E9" };
    std::fs::write(format!("{out_dir}/{stem_out}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/{stem_out}.json"), serde_json::to_string_pretty(&json!({"n": n, "task_a": ja, "task_b": jb, "runtime_s": t0.elapsed().as_secs_f64()})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}

/// Algorithm identity from a file stem: lowercase alphanumerics, with a few
/// well-known abbreviations expanded.
fn normalize_stem(stem: &str) -> String {
    let s: String = stem.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
    match s.as_str() {
        "bfs" => "breadthfirstsearch".into(),
        "dfs" => "depthfirstsearch".into(),
        _ => s,
    }
}
