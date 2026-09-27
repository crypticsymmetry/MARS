//! `mars-bench`: experiment runner.
//!
//! ```text
//! mars-bench sample [--groups N] [--naming canonical|synonyms|unresolved]
//! mars-bench e2 [--groups 1000] [--distractors 2]
//! mars-bench e0 [--groups 1000] [--seed 1] [--naming canonical] [--distractors 2] [--out results/E0] [--tag NAME]
//! ```

mod e0;
mod e2;
pub mod metrics;

use rustc_hash::FxHashMap;

/// `--key value` argument map.
pub struct Args {
    map: FxHashMap<String, String>,
}

impl Args {
    fn parse(raw: &[String]) -> Result<Self, String> {
        let mut map = FxHashMap::default();
        let mut i = 0;
        while i < raw.len() {
            let k = raw[i].strip_prefix("--").ok_or_else(|| format!("expected --key, got {}", raw[i]))?;
            let v = raw.get(i + 1).ok_or_else(|| format!("missing value for --{k}"))?;
            map.insert(k.to_string(), v.clone());
            i += 2;
        }
        Ok(Args { map })
    }
    pub fn str(&self, k: &str, default: &str) -> String {
        self.map.get(k).cloned().unwrap_or_else(|| default.to_string())
    }
    pub fn usize(&self, k: &str, default: usize) -> usize {
        self.map.get(k).map(|v| v.parse().expect("integer")).unwrap_or(default)
    }
    pub fn u64(&self, k: &str, default: u64) -> u64 {
        self.map.get(k).map(|v| v.parse().expect("integer")).unwrap_or(default)
    }
    pub fn f64(&self, k: &str, default: f64) -> f64 {
        self.map.get(k).map(|v| v.parse().expect("float")).unwrap_or(default)
    }
}

fn sample(args: &Args) -> Result<(), String> {
    use mars_gen::{generate, GenConfig, Naming};
    let naming = match args.str("naming", "canonical").as_str() {
        "synonyms" => Naming::Synonyms,
        "unresolved" => Naming::Unresolved,
        _ => Naming::Canonical,
    };
    let cfg = GenConfig { n_groups: args.usize("groups", 2), seed: args.u64("seed", 1), naming, ..Default::default() };
    let ds = generate(&cfg);
    for g in &ds.groups {
        println!(";; ===== group family={} =====", g.family.name());
        for (cls, i) in std::iter::once((mars_gen::VariantClass::Base, g.base)).chain(g.members()) {
            println!(";; {}", cls.name());
            println!("{}\n", ds.kb.render_case(ds.case(i)));
        }
    }
    Ok(())
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = argv.first() else {
        eprintln!("usage: mars-bench <sample|e0> [--key value ...]");
        std::process::exit(2);
    };
    let args = match Args::parse(&argv[1..]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };
    let r = match cmd.as_str() {
        "sample" => sample(&args),
        "e0" => e0::run(&args),
        "e2" => e2::run(&args),
        other => Err(format!("unknown command {other}")),
    };
    if let Err(e) = r {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
