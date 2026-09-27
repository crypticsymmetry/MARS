//! `mars`: command-line interface.
//!
//! ```text
//! mars analogies <files...> --case NAME [-k 5] [--profile analogy|literal]
//! mars map <files...> --base NAME --target NAME
//! mars stats <files...>
//! ```

use mars_encode::Profile;
use mars_engine::{Engine, EngineConfig, SqMode};
use mars_map::{Grounding, MapConfig, Mapper};
use mars_rel::{Kb, Term};
use std::process::exit;

struct Cli {
    files: Vec<String>,
    opts: Vec<(String, String)>,
}

impl Cli {
    fn parse(args: &[String]) -> Self {
        let (mut files, mut opts) = (Vec::new(), Vec::new());
        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            if let Some(k) = a.strip_prefix("--").or_else(|| a.strip_prefix('-')) {
                let v = args.get(i + 1).cloned().unwrap_or_default();
                opts.push((k.to_string(), v));
                i += 2;
            } else {
                files.push(a.clone());
                i += 1;
            }
        }
        Cli { files, opts }
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.opts.iter().rev().find(|x| x.0 == k).map(|x| x.1.as_str())
    }
    fn need(&self, k: &str) -> &str {
        self.get(k).unwrap_or_else(|| die(&format!("missing --{k}")))
    }
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    exit(2)
}

fn load(files: &[String]) -> Kb {
    if files.is_empty() {
        die("no input files");
    }
    let mut kb = Kb::new();
    for f in files {
        let src = std::fs::read_to_string(f).unwrap_or_else(|e| die(&format!("{f}: {e}")));
        kb.load_str(&src).unwrap_or_else(|e| die(&format!("{f}: {e}")));
    }
    kb
}

fn print_mapping(kb: &Kb, base: mars_rel::CaseId, target: mars_rel::CaseId) {
    let mp = Mapper::new(kb, MapConfig::default());
    let Some(m) = mp.best(base, target) else {
        println!("  (no structural mapping)");
        return;
    };
    println!("  structural score {:.2}", m.score);
    println!("  entity correspondences:");
    for (b, t) in &m.correspondences {
        if let (Term::Ent(x), Term::Ent(y)) = (b, t) {
            println!("    {} ↔ {}", kb.name(*x), kb.name(*y));
        }
    }
    println!("  matched relations:");
    for (b, t) in &m.correspondences {
        if let (Term::Expr(x), Term::Expr(_)) = (b, t) {
            if kb.case(base).facts.contains(x) {
                println!("    {}  ↔  {}", kb.render_term(*b), kb.render_term(*t));
            }
        }
    }
    let infs: Vec<_> = m.inferences.iter().filter(|i| i.grounding == Grounding::Structural).collect();
    if !infs.is_empty() {
        println!("  candidate inferences (hypotheses for the target):");
        for i in infs {
            println!("    {}   [support {:.2}{}]", mp.render_proj(&i.projected), i.support, if i.has_skolem { ", has skolem" } else { "" });
        }
    }
    if !m.alignable_differences.is_empty() {
        println!("  alignable differences:");
        for (b, t) in &m.alignable_differences {
            println!("    {}  vs  {}", kb.render_expr(*b), kb.render_expr(*t));
        }
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = argv.first().cloned() else {
        eprintln!("usage: mars <analogies|map|stats> <files.mars...> [options]");
        exit(2);
    };
    let cli = Cli::parse(&argv[1..]);
    match cmd.as_str() {
        "stats" => {
            let kb = load(&cli.files);
            println!("cases: {}, expressions: {}, predicates: {}, symbols: {}", kb.n_cases(), kb.n_exprs(), kb.vocab.len(), kb.interner.len());
        }
        "map" => {
            let kb = load(&cli.files);
            let b = kb.case_by_name(cli.need("base")).unwrap_or_else(|| die("unknown --base case"));
            let t = kb.case_by_name(cli.need("target")).unwrap_or_else(|| die("unknown --target case"));
            println!("{} → {}", cli.need("base"), cli.need("target"));
            print_mapping(&kb, b, t);
        }
        "analogies" => {
            let kb = load(&cli.files);
            let name = cli.need("case").to_string();
            let q = kb.case_by_name(&name).unwrap_or_else(|| die("unknown --case"));
            let k: usize = cli.get("k").unwrap_or("5").parse().unwrap_or(5);
            let profile = match cli.get("profile").unwrap_or("analogy") {
                "literal" => Profile::literal(),
                _ => Profile::analogy(),
            };
            let n = kb.n_cases();
            let mut e = Engine::new(kb, EngineConfig { profile, sq_mode: SqMode::Pipeline { mac_k: 64 }, slack: 8, ..Default::default() });
            let sq = e.add_standing_query(q, k.min(n.saturating_sub(1)).max(1));
            println!("analogues of {name} (fused = ½ structural + ½ fingerprint):");
            let top: Vec<_> = e.standing(sq).top().to_vec();
            for (rank, (c, s)) in top.iter().enumerate() {
                let fac = e.fac(q, *c);
                println!("  {}. {:<24} fused {:.3}  (structural {:.3})", rank + 1, e.kb.name(e.kb.case(*c).name), s, fac);
            }
            if let Some(&(best, _)) = top.first() {
                println!("\nbest analogue: {} → {name}", e.kb.name(e.kb.case(best).name));
                print_mapping(&e.kb, best, q);
                let infs = e.inferences(sq);
                if let Some(first) = infs.first() {
                    println!("\nprovenance of the first inference (JTMS):");
                    println!("  {}", e.explain(sq, first).unwrap_or_default().replace('\n', "\n  "));
                }
            }
        }
        other => die(&format!("unknown command {other}")),
    }
}
