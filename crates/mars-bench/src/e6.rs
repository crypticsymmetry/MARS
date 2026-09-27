//! E6: incremental maintenance (gate G5 / H3).
//!
//! A memory of N cases with Q standing queries receives a random stream of
//! updates (fact removals/re-additions/additions, case additions/removals,
//! biased toward cases in standing-query results). We measure per-update
//! work and time as N grows, and periodically check the incrementally
//! maintained results and inferences against a from-scratch recomputation.

use crate::metrics::mean;
use crate::Args;
use mars_engine::{Engine, EngineConfig, SqMode};
use mars_gen::{generate, GenConfig, Naming};
use mars_hv::Rng;
use mars_rel::{CaseId, CaseKind, ExprId, Term};
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Op {
    RemoveFact,
    ReAddFact,
    AddFact,
    AddCase,
    RemoveCase,
}

impl Op {
    fn name(self) -> &'static str {
        match self {
            Op::RemoveFact => "remove-fact",
            Op::ReAddFact => "re-add-fact",
            Op::AddFact => "add-fact",
            Op::AddCase => "add-case",
            Op::RemoveCase => "remove-case",
        }
    }
}

pub fn run(args: &Args) -> Result<(), String> {
    let groups = args.usize("groups", 1250);
    let reserve_groups = args.usize("reserve", 250);
    let n_sq = args.usize("sq", 200);
    let n_updates = args.usize("updates", 5000);
    let check_every = args.usize("check-every", 1000);
    let check_sample = args.usize("check-sample", 50);
    let k = args.usize("k", 5);
    let seed = args.u64("seed", 1);
    let out_dir = args.str("out", "results/E6");
    let tag = args.str("tag", &format!("{}-n{}", args.str("mode", "pipeline"), groups * 8));
    let t0 = Instant::now();

    let gcfg = GenConfig { seed, n_groups: groups + reserve_groups, naming: Naming::Canonical, distractors: 2, ..Default::default() };
    let mut ds = generate(&gcfg);
    // Standing-query cases: group bases with one root higher-order fact removed.
    let mut sq_cases = Vec::new();
    for g in 0..n_sq.min(groups) {
        let base = ds.case(ds.groups[g].base);
        let facts = ds.kb.case(base).facts.clone();
        let kb = &ds.kb;
        let nested: std::collections::HashSet<ExprId> =
            kb.case_exprs(base).into_iter().flat_map(|e| kb.expr(e).args.iter().filter_map(|a| if let Term::Expr(c) = *a { Some(c) } else { None }).collect::<Vec<_>>()).collect();
        let ho: Vec<ExprId> = facts.iter().copied().filter(|&f| kb.order(f) >= 2 && !nested.contains(&f)).collect();
        let drop = if ho.is_empty() { facts[0] } else { ho[0] };
        let kept: Vec<ExprId> = facts.into_iter().filter(|&f| f != drop).collect();
        sq_cases.push(ds.kb.add_case(&format!("sq{g}"), CaseKind::Query, kept));
    }
    let reserve: Vec<CaseId> = ds.items.iter().filter(|it| it.group >= groups).map(|it| it.case).collect();
    let fo_preds: Vec<mars_rel::Sym> = mars_gen::vocab::fo_predicates().iter().map(|p| ds.kb.interner.get(p).unwrap()).collect();
    let kb = ds.kb;
    let mode = match args.str("mode", "pipeline").as_str() {
        "exact" => SqMode::ExactFused,
        _ => SqMode::Pipeline { mac_k: args.usize("mac-k", 64) },
    };
    let mut engine = Engine::new(kb, EngineConfig { sq_mode: mode, ..Default::default() });
    for &r in &reserve {
        engine.remove_case(r);
    }
    let t_build = t0.elapsed();
    let sqs: Vec<usize> = sq_cases.iter().map(|&c| engine.add_standing_query(c, k)).collect();
    let t_sq = t0.elapsed() - t_build;
    let n_live = engine.n_live();
    eprintln!("[e6] engine: {n_live} live cases, {} standing queries ({:.1?}; SQ init {:.1?})", sqs.len(), t_build, t_sq);
    engine.work = Default::default();

    // Cost of recomputing every standing query from scratch (the non-incremental alternative).
    let t = Instant::now();
    let sample = sqs.len().min(20);
    for &sq in &sqs[..sample] {
        let _ = engine.scratch(sq);
    }
    let scratch_all_ms = t.elapsed().as_secs_f64() * 1e3 / sample as f64 * sqs.len() as f64;
    eprintln!("[e6] from-scratch recompute of all {} SQs ≈ {scratch_all_ms:.0} ms", sqs.len());

    let mut rng = Rng::new(seed ^ 0xE6);
    let mut removed: Vec<(CaseId, ExprId)> = Vec::new();
    let mut reserve_iter = reserve.into_iter();
    let mut per_op: Vec<(Op, f64, mars_engine::Work)> = Vec::new();
    let mut mismatches = 0usize;
    let mut mismatch_log: Vec<String> = Vec::new();
    let mut checks = 0usize;
    let n_cases_total = engine.kb.n_cases();
    let pick_case = |engine: &Engine, rng: &mut Rng| -> CaseId {
        // 30%: a case currently in some standing query's result ("hot").
        if rng.bernoulli(0.3) {
            let sq = rng.index(engine.n_standing());
            let top = engine.standing(sq).top();
            if !top.is_empty() {
                return top[rng.index(top.len())].0;
            }
        }
        loop {
            let c = CaseId(rng.index(engine.kb.n_cases()) as u32);
            if engine.is_alive(c) {
                return c;
            }
        }
    };
    let _ = n_cases_total;
    for u in 0..n_updates {
        let r = rng.f64();
        let op = if r < 0.35 {
            Op::RemoveFact
        } else if r < 0.60 {
            Op::ReAddFact
        } else if r < 0.70 {
            Op::AddFact
        } else if r < 0.90 {
            Op::AddCase
        } else {
            Op::RemoveCase
        };
        let before = engine.work.clone();
        let t = Instant::now();
        let mut done = op;
        match op {
            Op::RemoveFact => {
                let c = pick_case(&engine, &mut rng);
                let facts = engine.kb.case(c).facts.clone();
                if facts.len() > 1 {
                    let f = facts[rng.index(facts.len())];
                    engine.remove_fact(c, f);
                    removed.push((c, f));
                }
            }
            Op::ReAddFact => {
                if let Some((c, f)) = (!removed.is_empty()).then(|| removed.swap_remove(rng.index(removed.len()))) {
                    if engine.is_alive(c) {
                        engine.add_fact(c, f);
                    }
                }
            }
            Op::AddFact => {
                let c = pick_case(&engine, &mut rng);
                let ents = engine.kb.case_entities(c);
                if ents.len() >= 2 {
                    let ij = rng.sample_indices(ents.len(), 2);
                    let p = fo_preds[rng.index(fo_preds.len())];
                    let e = engine.kb.intern_expr(p, [Term::Ent(ents[ij[0]]), Term::Ent(ents[ij[1]])]);
                    engine.add_fact(c, e);
                }
            }
            Op::AddCase => match reserve_iter.next() {
                Some(res) => {
                    let facts = engine.kb.case(res).facts.clone();
                    engine.add_case(&format!("new{u}"), facts);
                }
                None => done = Op::RemoveFact,
            },
            Op::RemoveCase => {
                let c = pick_case(&engine, &mut rng);
                if !sq_cases.contains(&c) {
                    engine.remove_case(c);
                }
            }
        }
        let dt = t.elapsed().as_secs_f64() * 1e6;
        let w = &engine.work;
        let delta = mars_engine::Work {
            encodes: w.encodes - before.encodes,
            rows_written: w.rows_written - before.rows_written,
            sq_bound_checks: w.sq_bound_checks - before.sq_bound_checks,
            fac_evals: w.fac_evals - before.fac_evals,
            sq_full_recomputes: w.sq_full_recomputes - before.sq_full_recomputes,
            sq_incremental_changes: w.sq_incremental_changes - before.sq_incremental_changes,
            remaps: w.remaps - before.remaps,
            tms_touched: w.tms_touched - before.tms_touched,
        };
        per_op.push((done, dt, delta));
        engine.drain_events();

        if (u + 1) % check_every == 0 {
            // Exactness check against from-scratch recomputation (sampled SQs).
            let saved = engine.work.clone();
            for _ in 0..check_sample {
                let sq = rng.index(sqs.len());
                let inc_top: Vec<(CaseId, f64)> = engine.standing(sq).top().to_vec();
                let inc_inf = engine.inferences(sq);
                let (sc_top, sc_inf) = engine.scratch(sq);
                checks += 1;
                let same_top = inc_top.len() == sc_top.len() && inc_top.iter().zip(&sc_top).all(|(a, b)| (a.1 - b.1).abs() < 1e-9 && (a.0 == b.0 || (a.1 - b.1).abs() < 1e-12));
                if !same_top || inc_inf != sc_inf {
                    mismatches += 1;
                    if mismatches <= 5 {
                        let msg = format!("MISMATCH at update {} sq {sq}:\n inc     {inc_top:?}\n scratch {sc_top:?}\n inf inc {inc_inf:?}\n inf scr {sc_inf:?}\n", u + 1);
                        eprintln!("[e6] {msg}");
                        mismatch_log.push(msg);
                    }
                }
            }
            engine.work = saved;
            eprintln!("[e6] {} updates, {checks} checks, {mismatches} mismatches ({:.1?})", u + 1, t0.elapsed());
        }
    }

    // ---------------- Report
    let mut md = String::new();
    writeln!(md, "# E6: incremental maintenance ({tag})\n").unwrap();
    writeln!(md, "Mode {mode:?}. {n_live} live cases initially, {} standing queries (k = {k}), {n_updates} updates. Engine build {:.1?}; SQ initialization {:.1?}. **Exactness: {mismatches} mismatches in {checks} checks** against from-scratch recomputation.\n", sqs.len(), t_build, t_sq).unwrap();
    writeln!(md, "Recomputing all standing queries from scratch costs about **{scratch_all_ms:.0} ms** per update (the non-incremental alternative).\n").unwrap();
    writeln!(md, "| op | n | mean µs | p50 µs | p99 µs | encodes | SQ bound checks | FAC evals | SQ full recomputes | remaps | TMS touched |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|---|---|---|").unwrap();
    let mut jops = Vec::new();
    let ops = [Op::RemoveFact, Op::ReAddFact, Op::AddFact, Op::AddCase, Op::RemoveCase];
    let mut all_t = Vec::new();
    for op in ops {
        let rows: Vec<&(Op, f64, mars_engine::Work)> = per_op.iter().filter(|x| x.0 == op).collect();
        if rows.is_empty() {
            continue;
        }
        let mut ts: Vec<f64> = rows.iter().map(|x| x.1).collect();
        all_t.extend(ts.iter().copied());
        ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p = |q: f64| ts[((ts.len() as f64 * q) as usize).min(ts.len() - 1)];
        let m = |f: &dyn Fn(&mars_engine::Work) -> u64| mean(&rows.iter().map(|x| f(&x.2) as f64).collect::<Vec<_>>());
        let (enc, bc, fe, fr, rm, tt) = (m(&|w| w.encodes), m(&|w| w.sq_bound_checks), m(&|w| w.fac_evals), m(&|w| w.sq_full_recomputes), m(&|w| w.remaps), m(&|w| w.tms_touched));
        writeln!(md, "| {} | {} | {:.0} | {:.0} | {:.0} | {enc:.2} | {bc:.0} | {fe:.2} | {fr:.3} | {rm:.3} | {tt:.1} |", op.name(), rows.len(), mean(&ts), p(0.5), p(0.99)).unwrap();
        jops.push(json!({"op": op.name(), "n": rows.len(), "mean_us": mean(&ts), "p50_us": p(0.5), "p99_us": p(0.99), "encodes": enc, "sq_bound_checks": bc, "fac_evals": fe, "sq_full_recomputes": fr, "remaps": rm, "tms_touched": tt}));
    }
    all_t.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean_all = mean(&all_t);
    let p50 = all_t[all_t.len() / 2];
    writeln!(md, "\nAll updates: mean {mean_all:.0} µs, median {p50:.0} µs → **{:.0}× cheaper** (mean) than recomputing all standing queries per update.", scratch_all_ms * 1e3 / mean_all).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    if !mismatch_log.is_empty() {
        writeln!(md, "\n## Mismatch details\n\n```text\n{}```", mismatch_log.join("\n")).unwrap();
    }
    std::fs::write(format!("{out_dir}/E6-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let j = json!({"experiment": "E6", "tag": tag, "n_live": n_live, "n_sq": sqs.len(), "k": k, "n_updates": n_updates, "checks": checks, "mismatches": mismatches,
        "scratch_all_ms": scratch_all_ms, "mean_us": mean_all, "p50_us": p50, "ops": jops, "build_s": t_build.as_secs_f64(), "sq_init_s": t_sq.as_secs_f64()});
    std::fs::write(format!("{out_dir}/E6-{tag}.json"), serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
