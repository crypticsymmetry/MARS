//! E23: story analogies from natural language (StoryAnalogy multiple choice).
//!
//! Stories are converted to relational cases by an LLM front end
//! (`tools/llm2mars.py`: conceptual-dependency primitives + canonical
//! higher-order relations). Each of the 360 questions pairs a source story
//! with four choices: the analogy (*target*), a *noun* distractor sharing
//! surface content with the source, and two random stories. MARS scores
//! every (source, choice) pair; a method answers with its best-scoring
//! choice. Reported: accuracy and pairwise wins of the target over the noun
//! and random distractors. Also: the analogy score minus λ·surface-channel
//! similarity (Gentner: analogy = relational match without surface overlap).
//! Per-question scores are written to JSON so text
//! baselines (`tools/e23_baselines.py`) can be merged.

use crate::metrics::mean;
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_map::{MapConfig, Mapper};
use mars_rel::Kb;
use rayon::prelude::*;
use serde_json::json;
use std::fmt::Write as _;

const METHODS: [&str; 10] = ["fingerprint analogy profile", "fingerprint literal profile", "FAC (structural)", "fused 0.3·FAC + 0.7·FP-analogy", "fused 0.5·FAC + 0.5·FP-analogy", "FAC higher-order only", "analogy − 0.25·surface", "analogy − 0.5·surface", "analogy − 1·surface", "surface channel only (C0)"];

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/storyanalogy");
    let mc_path = args.str("mc", &format!("{dir}/storyanalogy_multiple_choice.json"));
    let out_dir = args.str("out", "results/E23");
    let mut kb = Kb::new();
    for f in ["vocab.mars", "cases.mars"] {
        kb.load_str(&std::fs::read_to_string(format!("{dir}/{f}")).map_err(|e| format!("{dir}/{f}: {e}"))?).map_err(|e| format!("{f}: {e}"))?;
    }
    let mc: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(&mc_path).map_err(|e| format!("{mc_path}: {e}"))?).map_err(|e| e.to_string())?;
    // Questions whose five stories were all converted.
    let questions: Vec<(usize, Vec<mars_rel::CaseId>, usize, Vec<String>)> = mc
        .iter()
        .enumerate()
        .filter_map(|(i, q)| {
            let s = kb.case_by_name(&format!("q{i}-s"))?;
            let n = q["choices"].as_array()?.len();
            let mut cs = vec![s];
            for j in 0..n {
                cs.push(kb.case_by_name(&format!("q{i}-c{j}"))?);
            }
            let types = q["types"].as_array()?.iter().map(|t| t.as_str().unwrap_or("").to_string()).collect();
            Some((i, cs, q["answer"].as_u64()? as usize, types))
        })
        .collect();
    let all: Vec<mars_rel::CaseId> = questions.iter().flat_map(|q| q.1.clone()).collect();
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let raw: Vec<Features> = all.par_iter().map(|&c| fx.extract(c)).collect();
    let stats = FeatureStats::fit(raw.iter());
    let sk = Sketcher::new(Layout::default(), 0xF1);
    let fps: Vec<_> = raw
        .into_par_iter()
        .map(|mut f| {
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f)
        })
        .collect();
    let idx_of: rustc_hash::FxHashMap<mars_rel::CaseId, usize> = all.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    let mapper = Mapper::new(&kb, MapConfig::default());
    let (an, li) = (Profile::analogy(), Profile::literal());
    // Higher-order-only mapping: structural score restricted to matches under canonical HO relations
    // is approximated by the mapper score of the HO facts of each case.
    let ho_names = ["cause", "enable", "prevent", "then", "despite", "repeat", "and"];
    let is_ho = |kb: &Kb, e: mars_rel::ExprId| ho_names.contains(&kb.name(kb.expr(e).functor));
    let mut kb_ho = kb.clone();
    let ho_case: rustc_hash::FxHashMap<mars_rel::CaseId, mars_rel::CaseId> = all
        .iter()
        .map(|&c| {
            let facts: Vec<mars_rel::ExprId> = kb.case(c).facts.iter().copied().filter(|&f| is_ho(&kb, f)).collect();
            let name = format!("ho:{}", kb.name(kb.case(c).name));
            (c, kb_ho.add_case(&name, mars_rel::CaseKind::Episode, facts))
        })
        .collect();
    let mapper_ho = Mapper::new(&kb_ho, MapConfig::default());
    let norm = |m: &Mapper, a: mars_rel::CaseId, b: mars_rel::CaseId| {
        let s = m.score(b, a) as f64;
        let (sa, sb) = (m.score(a, a) as f64, m.score(b, b) as f64);
        if s == 0.0 || sa == 0.0 || sb == 0.0 {
            0.0
        } else {
            (s / (sa * sb).sqrt()).min(1.0)
        }
    };
    let per_q: Vec<Vec<[f64; 10]>> = questions
        .par_iter()
        .map(|(_, cs, _, _)| {
            let s = cs[0];
            cs[1..]
                .iter()
                .map(|&c| {
                    let sims = sk.channel_sims(&fps[idx_of[&s]], &fps[idx_of[&c]]);
                    let (fa, fl) = (an.score(&sims), li.score(&sims));
                    let fac = norm(&mapper, s, c);
                    let fho = norm(&mapper_ho, ho_case[&s], ho_case[&c]);
                    // Gentner: an analogy is a relational match *without* surface
                    // (entity/attribute) overlap, so discount the surface channel C0.
                    let surf = sims[0];
                    [fa, fl, fac, 0.3 * fac + 0.7 * fa, 0.5 * fac + 0.5 * fa, fho, fa - 0.25 * surf, fa - 0.5 * surf, fa - surf, surf]
                })
                .collect()
        })
        .collect();

    let mut md = String::new();
    writeln!(md, "# E23: story analogies from natural language — MARS scores\n").unwrap();
    writeln!(md, "{} of {} StoryAnalogy multiple-choice questions fully converted (`tools/llm2mars.py`). Accuracy = target scored highest (chance 0.25; ties count as wrong); *target > noun* / *target > random* = pairwise wins over each distractor type.\n", questions.len(), mc.len()).unwrap();
    writeln!(md, "| method | accuracy | target > noun | target > random |\n|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    for (m, name) in METHODS.iter().enumerate() {
        let (mut acc, mut vn, mut vr) = (Vec::new(), Vec::new(), Vec::new());
        for ((_, _, ans, types), sc) in questions.iter().zip(&per_q) {
            let t = sc[*ans][m];
            acc.push(sc.iter().enumerate().all(|(j, x)| j == *ans || x[m] < t) as u8 as f64);
            for (j, ty) in types.iter().enumerate() {
                if j == *ans {
                    continue;
                }
                let win = (t > sc[j][m]) as u8 as f64;
                if ty == "noun" {
                    vn.push(win);
                } else if ty == "random" {
                    vr.push(win);
                }
            }
        }
        writeln!(md, "| {name} | {:.3} | {:.3} | {:.3} |", mean(&acc), mean(&vn), mean(&vr)).unwrap();
        rows.push(json!({"method": name, "accuracy": mean(&acc), "target_over_noun": mean(&vn), "target_over_random": mean(&vr)}));
    }
    let scores: Vec<serde_json::Value> = questions
        .iter()
        .zip(&per_q)
        .map(|((i, _, ans, types), sc)| json!({"q": i, "answer": ans, "types": types, "scores": METHODS.iter().enumerate().map(|(m, name)| (name.to_string(), json!(sc.iter().map(|x| x[m]).collect::<Vec<_>>()))).collect::<serde_json::Map<String, serde_json::Value>>()}))
        .collect();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E23-mars.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E23-mars.json"), serde_json::to_string_pretty(&json!({"questions": questions.len(), "rows": rows, "per_question": scores})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
