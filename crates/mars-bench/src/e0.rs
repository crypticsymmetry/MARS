//! E0: fingerprint separability and feature ablations (docs/EXPERIMENTS.md §4).
//!
//! For every group we score the base against each variant under many
//! methods and ask: does the true analogue (TA) outscore the mere-appearance
//! (MA), first-order-only (FOR) and random (RND) variants?

use crate::metrics::{auc, mean, std, win_rate};
use crate::Args;
use mars_encode::{
    channel_cosines, channel_sims, cosine, lexical_tokens, mac_content_vector, FeatureConfig, FeatureExtractor, FeatureStats, Features,
    Layout, Profile, Sketcher, SparseVec, N_CHANNELS,
};
use mars_gen::{generate, Dataset, Family, GenConfig, Naming, VariantClass};
use rayon::prelude::*;
use serde_json::{json, Value};
use std::fmt::Write as _;

/// Per-group scores: (class, score) for every non-base member.
type GroupScores = Vec<Vec<(VariantClass, f64)>>;

pub struct MethodResult {
    pub name: String,
    pub scores: GroupScores,
}

fn score_groups(ds: &Dataset, f: impl Fn(usize, usize) -> f64 + Sync) -> GroupScores {
    ds.groups
        .par_iter()
        .map(|g| g.members().into_iter().map(|(cls, i)| (cls, f(g.base, i))).collect())
        .collect()
}

#[derive(Default, Clone)]
pub struct Summary {
    pub auc_ma: f64,
    pub auc_for: f64,
    pub auc_rnd: f64,
    pub win_ma: f64,
    pub win_for: f64,
    pub ta_top: f64,
    pub n: usize,
}

fn summarize(ds: &Dataset, scores: &GroupScores, keep: impl Fn(Family) -> bool) -> Summary {
    summarize_groups(ds, scores, |g| keep(g.family))
}

fn summarize_groups(ds: &Dataset, scores: &GroupScores, keep: impl Fn(&mars_gen::Group) -> bool) -> Summary {
    let (mut ta, mut ma, mut fr, mut rnd) = (vec![], vec![], vec![], vec![]);
    let (mut p_ma, mut p_for, mut top) = (vec![], vec![], vec![]);
    for (g, s) in ds.groups.iter().zip(scores) {
        if !keep(g) {
            continue;
        }
        let get = |c: VariantClass| s.iter().filter(|x| x.0 == c).map(|x| x.1).collect::<Vec<_>>();
        let (t, m, f, r) = (get(VariantClass::TA)[0], get(VariantClass::MA)[0], get(VariantClass::FOR)[0], get(VariantClass::RND));
        ta.push(t);
        ma.push(m);
        fr.push(f);
        rnd.extend(&r);
        p_ma.push((t, m));
        p_for.push((t, f));
        let best_other = r.iter().copied().fold(m.max(f), f64::max);
        top.push((t, best_other));
    }
    Summary {
        auc_ma: auc(&ta, &ma),
        auc_for: auc(&ta, &fr),
        auc_rnd: auc(&ta, &rnd),
        win_ma: win_rate(&p_ma),
        win_for: win_rate(&p_for),
        ta_top: win_rate(&top),
        n: ta.len(),
    }
}

fn summary_json(s: &Summary) -> Value {
    json!({"n": s.n, "auc_ta_vs_ma": s.auc_ma, "auc_ta_vs_for": s.auc_for, "auc_ta_vs_rnd": s.auc_rnd,
           "win_ta_gt_ma": s.win_ma, "win_ta_gt_for": s.win_for, "ta_top1": s.ta_top})
}

fn extract_all(ds: &Dataset, cfg: &FeatureConfig) -> Vec<Features> {
    let fx = FeatureExtractor::new(&ds.kb, cfg.clone());
    ds.items.par_iter().map(|it| fx.extract(it.case)).collect()
}

fn with_idf(feats: &[Features]) -> Vec<Features> {
    let stats = FeatureStats::fit(feats.iter());
    feats
        .par_iter()
        .map(|f| {
            let mut g = f.clone();
            stats.apply(&mut g, &[true; N_CHANNELS]);
            g
        })
        .collect()
}

fn tfidf(vs: &[SparseVec]) -> Vec<SparseVec> {
    let feats: Vec<Features> = vs.iter().map(|v| Features { channels: [v.clone(), vec![], vec![], vec![], vec![]] }).collect();
    let stats = FeatureStats::fit(feats.iter());
    vs.iter()
        .map(|v| {
            let mut v = v.clone();
            stats.apply_sparse(0, &mut v);
            v
        })
        .collect()
}

pub fn run(args: &Args) -> Result<(), String> {
    let groups = args.usize("groups", 1000);
    let seed = args.u64("seed", 1);
    let naming = match args.str("naming", "canonical").as_str() {
        "canonical" => Naming::Canonical,
        "synonyms" => Naming::Synonyms,
        "unresolved" => Naming::Unresolved,
        other => return Err(format!("unknown naming {other}")),
    };
    let distractors = args.usize("distractors", 2);
    let (ops, severity) = args.perturbation();
    let out_dir = args.str("out", "results/E0");
    let tag = args.str("tag", &format!("{}-d{}", args.str("naming", "canonical"), distractors));

    let t0 = std::time::Instant::now();
    let gcfg = GenConfig { seed, n_groups: groups, naming, distractors, perturb_ops: ops, severity, ..Default::default() };
    let ds = generate(&gcfg);
    eprintln!("[e0] generated {} cases in {:.2?}", ds.items.len(), t0.elapsed());

    let base_cfg = FeatureConfig::default();
    let raw = extract_all(&ds, &base_cfg);
    let idf = with_idf(&raw);
    let n_feat: Vec<f64> = raw.iter().map(|f| f.len() as f64).collect();
    eprintln!("[e0] features: mean {:.1} per case ({:.2?})", mean(&n_feat), t0.elapsed());

    let analogy = Profile::analogy();
    let mut methods: Vec<MethodResult> = Vec::new();
    let mut add = |name: &str, scores: GroupScores| methods.push(MethodResult { name: name.to_string(), scores });

    // ---- Baselines
    let lex = tfidf(&ds.items.iter().map(|it| lexical_tokens(&ds.kb, it.case)).collect::<Vec<_>>());
    add("B2 lexical TF-IDF", score_groups(&ds, |a, b| cosine(&lex[a], &lex[b])));
    let mac: Vec<SparseVec> = ds.items.iter().map(|it| mac_content_vector(&ds.kb, it.case)).collect();
    add("B4 MAC content vectors", score_groups(&ds, |a, b| cosine(&mac[a], &mac[b])));

    // ---- Exact channel cosines
    for (c, label) in ["C0 surface", "C1 content", "C2 relational", "C3 WL", "C4 topology"].iter().enumerate() {
        add(&format!("exact {label}"), score_groups(&ds, |a, b| cosine(&raw[a].channels[c], &raw[b].channels[c])));
    }
    add("exact analogy profile", score_groups(&ds, |a, b| analogy.score(&channel_cosines(&raw[a], &raw[b]))));
    add("exact analogy profile +IDF", score_groups(&ds, |a, b| analogy.score(&channel_cosines(&idf[a], &idf[b]))));
    add("exact literal profile +IDF", score_groups(&ds, |a, b| Profile::literal().score(&channel_cosines(&idf[a], &idf[b]))));

    // ---- Fingerprints (D sweep, IDF)
    let mut fp8192 = None;
    for total in [1024usize, 2048, 4096, 8192, 16384] {
        let sk = Sketcher::new(Layout::scaled(total), seed ^ 0xF1);
        let fps: Vec<Vec<u64>> = idf.par_iter().map(|f| sk.sketch(f).into_words()).collect();
        let layout = sk.layout.clone();
        add(&format!("fingerprint analogy +IDF D={total}"), score_groups(&ds, |a, b| analogy.score(&channel_sims(&layout, &fps[a], &fps[b]))));
        if total == 8192 {
            fp8192 = Some((layout, fps));
        }
    }
    let (layout, fps) = fp8192.unwrap();
    for (c, label) in ["C1", "C2", "C3", "C4"].iter().enumerate() {
        let c = c + 1;
        add(&format!("fingerprint {label} only D=8192"), score_groups(&ds, |a, b| channel_sims(&layout, &fps[a], &fps[b])[c]));
    }
    eprintln!("[e0] fingerprints done ({:.2?})", t0.elapsed());

    // ---- Channel-mix ablations on exact+IDF
    let mixes: [(&str, [f64; N_CHANNELS]); 6] = [
        ("mix C2 only", [0.0, 0.0, 1.0, 0.0, 0.0]),
        ("mix C3 only", [0.0, 0.0, 0.0, 1.0, 0.0]),
        ("mix C4 only", [0.0, 0.0, 0.0, 0.0, 1.0]),
        ("mix C2+C3", [0.0, 0.0, 0.5, 0.5, 0.0]),
        ("mix C2+C3+C4", [0.0, 0.0, 0.4, 0.4, 0.2]),
        ("mix C1+C2+C3", [0.0, 0.2, 0.4, 0.4, 0.0]),
    ];
    for (name, w) in mixes {
        let p = Profile::new(name, w);
        add(&format!("exact {name} +IDF"), score_groups(&ds, |a, b| p.score(&channel_cosines(&idf[a], &idf[b]))));
    }

    // ---- Feature-config ablations (exact analogy profile +IDF)
    let ablations: Vec<(&str, FeatureConfig)> = vec![
        ("no taxonomy", FeatureConfig { taxonomy_alpha: 0.0, ..base_cfg.clone() }),
        ("no systematicity (beta=0)", FeatureConfig { beta: 0.0, ..base_cfg.clone() }),
        ("no parent-child", FeatureConfig { parent_child: false, ..base_cfg.clone() }),
        ("no co-entity", FeatureConfig { co_entity: false, ..base_cfg.clone() }),
        ("co-entity weight 1.0", FeatureConfig { co_entity_weight: 1.0, ..base_cfg.clone() }),
        ("co-entity weight 0.5", FeatureConfig { co_entity_weight: 0.5, ..base_cfg.clone() }),
        ("no co-expr", FeatureConfig { co_expr: false, ..base_cfg.clone() }),
        ("WL depth 1", FeatureConfig { wl_depth: 1, ..base_cfg.clone() }),
        ("WL depth 3", FeatureConfig { wl_depth: 3, ..base_cfg.clone() }),
    ];
    for (name, cfg) in ablations {
        let f = with_idf(&extract_all(&ds, &cfg));
        add(&format!("ablation: {name}"), score_groups(&ds, |a, b| analogy.score(&channel_cosines(&f[a], &f[b]))));
    }
    eprintln!("[e0] ablations done ({:.2?})", t0.elapsed());

    // ---- Distance distribution per class (fingerprint D=8192)
    let mut dist_rows = Vec::new();
    for cls in [VariantClass::LS, VariantClass::TA, VariantClass::MA, VariantClass::FOR, VariantClass::RND] {
        let mut per_ch: [Vec<f64>; N_CHANNELS] = Default::default();
        for g in &ds.groups {
            for (c2, i) in g.members() {
                if c2 == cls {
                    let s = channel_sims(&layout, &fps[g.base], &fps[i]);
                    for c in 0..N_CHANNELS {
                        per_ch[c].push((1.0 - s[c]) / 2.0);
                    }
                }
            }
        }
        dist_rows.push((cls, per_ch.map(|v| (mean(&v), std(&v)))));
    }

    // ---- Report
    let is_dev = |f: Family| !f.is_test();
    let is_test = |f: Family| f.is_test();
    let mut md = String::new();
    writeln!(md, "# E0 — fingerprint separability ({tag})\n").unwrap();
    writeln!(
        md,
        "Config: groups={groups}, seed={seed}, naming={:?}, distractors={distractors}, cases={}, mean features/case={:.1}, runtime={:.1?}.\n",
        naming,
        ds.items.len(),
        mean(&n_feat),
        t0.elapsed()
    )
    .unwrap();
    writeln!(md, "Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).\n").unwrap();
    let n_disc = ds.groups.iter().filter(|g| g.discriminable(&ds)).count();
    writeln!(md, "Discriminable groups (TA preserves more base higher-order facts than MA and FOR): {n_disc}/{}.\n", ds.groups.len()).unwrap();
    writeln!(md, "| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|---|---|").unwrap();
    let mut jmethods = Vec::new();
    for m in &methods {
        let all = summarize(&ds, &m.scores, |_| true);
        let dev = summarize(&ds, &m.scores, is_dev);
        let test = summarize(&ds, &m.scores, is_test);
        let disc = summarize_groups(&ds, &m.scores, |g| g.discriminable(&ds));
        writeln!(
            md,
            "| {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |",
            m.name, all.auc_ma, all.auc_for, all.auc_rnd, all.win_ma, all.win_for, all.ta_top, dev.ta_top, test.ta_top, disc.ta_top
        )
        .unwrap();
        let fams: Vec<Value> = Family::ALL
            .iter()
            .map(|&f| json!({"family": f.name(), "summary": summary_json(&summarize(&ds, &m.scores, |x| x == f))}))
            .collect();
        jmethods.push(json!({"method": m.name, "all": summary_json(&all), "dev": summary_json(&dev), "test": summary_json(&test), "discriminable": summary_json(&disc), "per_family": fams}));
    }

    // Per-family breakdown for the headline methods.
    let headline = ["B4 MAC content vectors", "exact analogy profile +IDF", "fingerprint analogy +IDF D=8192"];
    writeln!(md, "\n## TA-top by family\n").unwrap();
    write!(md, "| family |").unwrap();
    for h in headline {
        write!(md, " {h} |").unwrap();
    }
    writeln!(md, "\n|---|{}", "---|".repeat(headline.len())).unwrap();
    for f in Family::ALL {
        write!(md, "| {}{} |", f.name(), if f.is_test() { " (test)" } else { "" }).unwrap();
        for h in headline {
            let m = methods.iter().find(|m| m.name == h).unwrap();
            write!(md, " {:.3} |", summarize(&ds, &m.scores, |x| x == f).ta_top).unwrap();
        }
        writeln!(md).unwrap();
    }

    writeln!(md, "\n## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)\n").unwrap();
    writeln!(md, "| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|").unwrap();
    let mut jdist = Vec::new();
    for (cls, d) in &dist_rows {
        writeln!(
            md,
            "| {} | {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} |",
            cls.name(), d[0].0, d[0].1, d[1].0, d[1].1, d[2].0, d[2].1, d[3].0, d[3].1, d[4].0, d[4].1
        )
        .unwrap();
        jdist.push(json!({"class": cls.name(), "mean": d.map(|x| x.0), "sd": d.map(|x| x.1)}));
    }

    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let md_path = format!("{out_dir}/E0-{tag}.md");
    let json_path = format!("{out_dir}/E0-{tag}.json");
    std::fs::write(&md_path, &md).map_err(|e| e.to_string())?;
    let j = json!({
        "experiment": "E0", "tag": tag, "gen_config": gcfg, "feature_config": base_cfg,
        "layout": layout, "methods": jmethods, "distances": jdist, "n_discriminable": n_disc,
        "runtime_s": t0.elapsed().as_secs_f64(),
    });
    std::fs::write(&json_path, serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    eprintln!("[e0] wrote {md_path} and {json_path}");
    Ok(())
}
