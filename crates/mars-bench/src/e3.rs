//! E3: end-to-end MAC → FAC retrieval at scale (docs/EXPERIMENTS.md §4).
//!
//! Corpus: every case of G generated groups (8·G cases). Queries: the base
//! cases of the first Q groups. For each query the base itself and its LS
//! (a legitimate literal match) are excluded; the target is the group's TA.
//!
//! MAC candidates come from the fingerprint index (Mode K) or from exact
//! sparse baselines; FAC re-ranks the top-k by normalized structural score or
//! by the fused score (½ FAC + ½ fingerprint). Exhaustive FAC over the whole
//! corpus is the upper bound (sampled queries).

use crate::metrics::{mean, mrr, recall_at};
use crate::Args;
use mars_encode::{lexical_tokens, mac_content_vector, FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, SparseVec, N_CHANNELS};
use mars_gen::{generate, GenConfig, Naming};
use mars_index::{Hit, ModeK, MultiSparse, SparseIndex};
use mars_map::{MapConfig, Mapper};
use rayon::prelude::*;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::sync::OnceLock;
use std::time::Instant;

const NOT_FOUND: usize = usize::MAX / 2;

struct QueryOut {
    /// Rank of TA in the MAC list (after exclusions), NOT_FOUND if absent.
    mac_rank: usize,
    ma_above: bool,
    for_above: bool,
    /// Rank of TA after FAC re-rank (structural only / fused).
    fac_rank: usize,
    fused_rank: usize,
    fac_us: f64,
}

pub fn run(args: &Args) -> Result<(), String> {
    let groups = args.usize("groups", 1250);
    let n_queries = args.usize("queries", 1000).min(groups);
    let k = args.usize("k", 64);
    let seed = args.u64("seed", 1);
    let distractors = args.usize("distractors", 2);
    let (ops, severity) = args.perturbation();
    let baselines = args.str("baselines", "auto");
    let exhaustive_q = args.usize("exhaustive", 50);
    let out_dir = args.str("out", "results/E3");
    let tag = args.str("tag", &format!("n{}-s{severity}", groups * 8));
    let t0 = Instant::now();

    let gcfg = GenConfig { seed, n_groups: groups, naming: Naming::Canonical, distractors, perturb_ops: ops, severity, ..Default::default() };
    let ds = generate(&gcfg);
    let n = ds.items.len();
    eprintln!("[e3] generated {n} cases ({:.1?})", t0.elapsed());
    let run_baselines = match baselines.as_str() {
        "auto" => n <= 200_000,
        "yes" | "true" => true,
        _ => false,
    };

    // ---------------- Encode
    let fx = FeatureExtractor::new(&ds.kb, FeatureConfig::default());
    let mut feats: Vec<Features> = ds.items.par_iter().map(|it| fx.extract(it.case)).collect();
    let stats = FeatureStats::fit(feats.iter());
    feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
    let t_feat = t0.elapsed();
    let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);
    let fps: Vec<Vec<u64>> = feats.par_iter().map(|f| sk.sketch(f).into_words()).collect();
    let mut idx = ModeK::with_capacity(sk.layout.clone(), n);
    for f in &fps {
        idx.push(f);
    }
    let t_index = t0.elapsed();
    eprintln!("[e3] features {:.1?}, fingerprints+index {:.1?}", t_feat, t_index);
    let profile = Profile::analogy();
    let scorer = idx.scorer(&profile.weights);

    let queries: Vec<usize> = (0..n_queries).map(|g| ds.groups[g].base).collect();
    let excluded = |qi: usize, id: u32| -> bool {
        let g = &ds.groups[qi];
        id as usize == g.base || id as usize == g.ls
    };
    let filter = |qi: usize, hits: Vec<Hit>, k: usize| -> Vec<Hit> { hits.into_iter().filter(|h| !excluded(qi, h.id)).take(k).collect() };

    // ---------------- MAC: fingerprint Mode K (batched)
    let k_eval = 256usize.max(k);
    let t_mac = Instant::now();
    let mut fp_lists: Vec<Vec<Hit>> = Vec::with_capacity(n_queries);
    for chunk in queries.chunks(64) {
        let qs: Vec<&[u64]> = chunk.iter().map(|&q| fps[q].as_slice()).collect();
        fp_lists.extend(idx.search_batch(&qs, &scorer, k_eval + 2));
    }
    let mac_ms = t_mac.elapsed().as_secs_f64() * 1e3 / n_queries as f64;
    let fp_lists: Vec<Vec<Hit>> = fp_lists.into_iter().enumerate().map(|(qi, l)| filter(qi, l, k_eval)).collect();
    eprintln!("[e3] Mode K: {mac_ms:.2} ms/query ({:.1?})", t0.elapsed());

    // ---------------- Baseline MAC lists (exact sparse)
    let mut baseline_lists: Vec<(String, Vec<Vec<Hit>>, f64)> = Vec::new();
    if run_baselines {
        let mac_vecs: Vec<SparseVec> = ds.items.par_iter().map(|it| mac_content_vector(&ds.kb, it.case)).collect();
        let lex_raw: Vec<SparseVec> = ds.items.par_iter().map(|it| lexical_tokens(&ds.kb, it.case)).collect();
        let lex_stats = FeatureStats::fit(lex_raw.iter().map(|v| Features { channels: [v.clone(), vec![], vec![], vec![], vec![]] }).collect::<Vec<_>>().iter());
        let lex: Vec<SparseVec> = lex_raw
            .into_iter()
            .map(|mut v| {
                lex_stats.apply_sparse(0, &mut v);
                v
            })
            .collect();
        let chans: Vec<(f32, SparseIndex)> = (0..N_CHANNELS)
            .map(|c| (profile.weights[c] as f32, SparseIndex::build(&feats.iter().map(|f| f.channels[c].clone()).collect::<Vec<_>>())))
            .collect();
        let specs: Vec<(&str, MultiSparse, Vec<Vec<&SparseVec>>)> = vec![
            ("B2 lexical TF-IDF", MultiSparse { channels: vec![(1.0, SparseIndex::build(&lex))] }, queries.iter().map(|&q| vec![&lex[q]]).collect()),
            ("B4 MAC content vectors", MultiSparse { channels: vec![(1.0, SparseIndex::build(&mac_vecs))] }, queries.iter().map(|&q| vec![&mac_vecs[q]]).collect()),
            ("B5 exact sparse (analogy profile)", MultiSparse { channels: chans }, queries.iter().map(|&q| feats[q].channels.iter().collect()).collect()),
        ];
        for (name, ms, qvecs) in specs {
            let t = Instant::now();
            let lists = ms.search_batch(&qvecs, k_eval + 2);
            let ms_per = t.elapsed().as_secs_f64() * 1e3 / n_queries as f64;
            let lists: Vec<Vec<Hit>> = lists.into_iter().enumerate().map(|(qi, l)| filter(qi, l, k_eval)).collect();
            eprintln!("[e3] baseline {name}: {ms_per:.2} ms/query");
            baseline_lists.push((name.to_string(), lists, ms_per));
        }
    }
    drop(feats);

    // ---------------- FAC re-rank
    let mapper = Mapper::new(&ds.kb, MapConfig::default());
    let self_scores: Vec<OnceLock<f32>> = (0..n).map(|_| OnceLock::new()).collect();
    let self_score = |i: usize| *self_scores[i].get_or_init(|| mapper.score(ds.case(i), ds.case(i)));
    let fac_norm = |q: usize, c: usize| -> f64 {
        let s = mapper.score(ds.case(q), ds.case(c)) as f64;
        if s == 0.0 {
            0.0
        } else {
            s / ((self_score(q) as f64) * (self_score(c) as f64)).sqrt()
        }
    };
    let fp_sim = |q: usize, c: usize| -> f64 { idx.score(&scorer, &fps[q], c as u32) as f64 };

    let rerank = |qi: usize, list: &[Hit]| -> (usize, usize, f64) {
        let g = &ds.groups[qi];
        let q = g.base;
        let t = Instant::now();
        let mut scored: Vec<(usize, f64, f64)> = list[..list.len().min(k)].iter().map(|h| {
            let c = h.id as usize;
            let f = fac_norm(q, c);
            (c, f, 0.5 * f + 0.5 * fp_sim(q, c))
        }).collect();
        let us = t.elapsed().as_secs_f64() * 1e6;
        let rank_by = |scored: &mut Vec<(usize, f64, f64)>, key: fn(&(usize, f64, f64)) -> f64| -> usize {
            scored.sort_by(|a, b| key(b).total_cmp(&key(a)).then(a.0.cmp(&b.0)));
            scored.iter().position(|x| x.0 == g.ta).unwrap_or(NOT_FOUND)
        };
        let fac_r = rank_by(&mut scored, |x| x.1);
        let fused_r = rank_by(&mut scored, |x| x.2);
        (fac_r, fused_r, us)
    };

    let eval_list = |lists: &[Vec<Hit>]| -> Vec<QueryOut> {
        lists
            .par_iter()
            .enumerate()
            .map(|(qi, l)| {
                let g = &ds.groups[qi];
                let pos = |id: usize| l.iter().position(|h| h.id as usize == id).unwrap_or(NOT_FOUND);
                let mac_rank = pos(g.ta);
                let (fac_rank, fused_rank, fac_us) = rerank(qi, l);
                QueryOut { mac_rank, ma_above: pos(g.ma) < mac_rank, for_above: pos(g.for_) < mac_rank, fac_rank, fused_rank, fac_us }
            })
            .collect()
    };

    let mut pipelines: Vec<(String, Vec<QueryOut>, f64)> = vec![("fingerprint Mode K".into(), eval_list(&fp_lists), mac_ms)];
    for (name, lists, ms) in &baseline_lists {
        pipelines.push((name.clone(), eval_list(lists), *ms));
    }
    eprintln!("[e3] FAC re-rank done ({:.1?})", t0.elapsed());

    // ---------------- Exhaustive FAC upper bound (sampled queries)
    let ex_q = exhaustive_q.min(n_queries);
    let t_ex = Instant::now();
    let exhaustive: Vec<(usize, usize)> = (0..ex_q)
        .into_par_iter()
        .map(|qi| {
            let g = &ds.groups[qi];
            let mut all: Vec<(usize, f64, f64)> = (0..n)
                .filter(|&c| !excluded(qi, c as u32))
                .map(|c| {
                    let f = fac_norm(g.base, c);
                    (c, f, 0.5 * f + 0.5 * fp_sim(g.base, c))
                })
                .collect();
            all.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let fr = all.iter().position(|x| x.0 == g.ta).unwrap();
            all.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
            let ur = all.iter().position(|x| x.0 == g.ta).unwrap();
            (fr, ur)
        })
        .collect();
    let ex_ms = t_ex.elapsed().as_secs_f64() * 1e3 * rayon::current_num_threads() as f64 / ex_q.max(1) as f64;
    eprintln!("[e3] exhaustive FAC on {ex_q} queries ({:.1?})", t0.elapsed());

    // ---------------- Report
    let disc: Vec<bool> = (0..n_queries).map(|qi| ds.groups[qi].discriminable(&ds)).collect();
    let n_disc = disc.iter().filter(|&&d| d).count();
    let mut md = String::new();
    writeln!(md, "# E3: end-to-end MAC → FAC ({tag})\n").unwrap();
    writeln!(md, "Corpus {n} cases ({groups} groups), {n_queries} queries (group bases; base and LS excluded; target = TA), k = {k}, distractors = {distractors}, perturbation = {:?} × {severity}. Discriminable queries: {n_disc}/{n_queries}. Threads: {}. Total runtime {:.1?}.\n", gcfg.perturb_ops.iter().map(|o| o.name()).collect::<Vec<_>>(), rayon::current_num_threads(), t0.elapsed()).unwrap();
    writeln!(md, "| MAC stage | MAC ms/query | R@1 | R@10 | R@{k} | R@256 | MRR | MA above TA | FOR above TA | FAC acc@1 | **fused acc@1** | fused acc@1 (discr.) | FAC µs/query (1 thread) |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|---|---|---|---|---|").unwrap();
    let mut jp: Vec<Value> = Vec::new();
    for (name, outs, ms) in &pipelines {
        let ranks: Vec<usize> = outs.iter().map(|o| o.mac_rank).collect();
        let acc = |f: &dyn Fn(&QueryOut) -> bool, only_disc: bool| -> f64 {
            let v: Vec<f64> = outs.iter().zip(&disc).filter(|(_, &d)| !only_disc || d).map(|(o, _)| f(o) as u8 as f64).collect();
            mean(&v)
        };
        let fac1 = acc(&|o| o.fac_rank == 0, false);
        let fused1 = acc(&|o| o.fused_rank == 0, false);
        let fused1d = acc(&|o| o.fused_rank == 0, true);
        let ma = acc(&|o| o.ma_above, false);
        let fo = acc(&|o| o.for_above, false);
        let fac_us = mean(&outs.iter().map(|o| o.fac_us).collect::<Vec<_>>());
        writeln!(
            md,
            "| {name} | {ms:.2} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {ma:.3} | {fo:.3} | {fac1:.3} | **{fused1:.3}** | {fused1d:.3} | {fac_us:.0} |",
            recall_at(&ranks, 1), recall_at(&ranks, 10), recall_at(&ranks, k), recall_at(&ranks, 256), mrr(&ranks)
        )
        .unwrap();
        jp.push(json!({"mac": name, "mac_ms_per_query": ms, "r1": recall_at(&ranks, 1), "r10": recall_at(&ranks, 10), "rk": recall_at(&ranks, k),
            "r256": recall_at(&ranks, 256), "mrr": mrr(&ranks), "ma_above": ma, "for_above": fo, "fac_acc1": fac1, "fused_acc1": fused1,
            "fused_acc1_discriminable": fused1d, "fac_us_per_query": fac_us}));
    }
    let ex_fac1 = mean(&exhaustive.iter().map(|x| (x.0 == 0) as u8 as f64).collect::<Vec<_>>());
    let ex_fused1 = mean(&exhaustive.iter().map(|x| (x.1 == 0) as u8 as f64).collect::<Vec<_>>());
    let fp_fused_on_same = {
        let o = &pipelines[0].1;
        mean(&o[..ex_q].iter().map(|x| (x.fused_rank == 0) as u8 as f64).collect::<Vec<_>>())
    };
    writeln!(md, "\n**Exhaustive FAC upper bound** (first {ex_q} queries, mapper over all {} cases): FAC acc@1 {ex_fac1:.3}, fused acc@1 {ex_fused1:.3}; the Mode K → fused pipeline on the same queries: {fp_fused_on_same:.3}. Exhaustive cost ≈ {ex_ms:.0} ms/query single-threaded vs {:.1} ms for Mode K + FAC@{k} (fraction of corpus mapped: {:.2e}).", n - 2, mac_ms + mean(&pipelines[0].1.iter().map(|o| o.fac_us).collect::<Vec<_>>()) / 1e3, k as f64 / n as f64).unwrap();

    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E3-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let j = json!({"experiment": "E3", "tag": tag, "gen_config": gcfg, "n_cases": n, "n_queries": n_queries, "k": k, "n_discriminable": n_disc,
        "pipelines": jp, "exhaustive": {"n_queries": ex_q, "fac_acc1": ex_fac1, "fused_acc1": ex_fused1, "modek_fused_same_queries": fp_fused_on_same, "ms_per_query_1thread": ex_ms},
        "timing": {"features_s": t_feat.as_secs_f64(), "index_s": t_index.as_secs_f64(), "total_s": t0.elapsed().as_secs_f64()}});
    std::fs::write(format!("{out_dir}/E3-{tag}.json"), serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}

