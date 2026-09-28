//! Persistence: snapshot + frozen IDF epoch + append-only operation log.
//!
//! A store directory holds
//! * `kb.mars`   — vocabulary and every case (in `CaseId` order, so ids are stable),
//! * `meta.txt`  — engine metadata: `retire <case>` and `standing <case> <k>` lines,
//! * `epoch.idf` — the frozen feature statistics (exact reproducibility),
//! * `identity.idf` — the identity channel's frozen statistics (when enabled),
//! * `log.txt`   — operations since the snapshot, one per line, replayed on open.
//!
//! Log records: `declare <defpredicate…>`, `add-case <defcase…>`,
//! `add-fact <case> <expr>`, `remove-fact <case> <expr>`, `remove-case <case>`,
//! `standing <case> <k>`, `transfer-outcome correct|wrong <type>...` (feedback).
//! Meta also holds the learned transfer counts (`transfer-total`, `transfer-counts`).

use crate::{Engine, EngineConfig};
use mars_encode::FeatureStats;
use mars_rel::{sexpr, CaseId, Kb};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn io<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

impl Engine {
    /// Start appending every mutation to `path`.
    pub fn attach_log(&mut self, path: &Path) -> Result<(), String> {
        let f = OpenOptions::new().create(true).append(true).open(path).map_err(io)?;
        self.log = Some(BufWriter::new(f));
        Ok(())
    }

    pub(crate) fn log_line(&mut self, line: String) {
        if self.replaying {
            return;
        }
        if let Some(w) = self.log.as_mut() {
            // Durability over throughput: flush every record.
            let _ = writeln!(w, "{}", line.replace('\n', " "));
            let _ = w.flush();
        }
    }

    fn case_name(&self, c: CaseId) -> String {
        self.kb.name(self.kb.case(c).name).to_string()
    }

    /// Write a snapshot of the whole engine into `dir` and truncate the log.
    pub fn checkpoint(&mut self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(io)?;
        let mut kb_text = self.kb.render_vocab();
        for c in 0..self.kb.n_cases() {
            kb_text.push_str(&self.kb.render_case(CaseId(c as u32)));
            kb_text.push('\n');
        }
        let mut meta = String::new();
        for c in 0..self.kb.n_cases() {
            if !self.alive[c] {
                meta.push_str(&format!("retire {}\n", self.case_name(CaseId(c as u32))));
            }
        }
        for sq in &self.sqs {
            meta.push_str(&format!("standing {} {}\n", self.kb.name(self.kb.case(sq.case).name), sq.k));
        }
        if !self.transfers.is_empty() {
            meta.push_str(&self.transfers.to_meta());
        }
        // Write-then-rename for atomicity of each file.
        let write = |name: &str, bytes: &[u8]| -> Result<(), String> {
            let tmp = dir.join(format!("{name}.tmp"));
            std::fs::write(&tmp, bytes).map_err(io)?;
            std::fs::rename(&tmp, dir.join(name)).map_err(io)
        };
        write("kb.mars", kb_text.as_bytes())?;
        write("meta.txt", meta.as_bytes())?;
        write("epoch.idf", &self.stats.to_bytes())?;
        if let Some(st) = self.identity_stats() {
            write("identity.idf", &st.to_bytes())?;
        }
        write("log.txt", b"")?;
        self.log = None;
        self.attach_log(&dir.join("log.txt"))
    }

    /// Open a store: load the snapshot with its frozen epoch, replay the log,
    /// and keep logging to it.
    pub fn open(dir: &Path, cfg: EngineConfig) -> Result<Engine, String> {
        let mut kb = Kb::new();
        kb.load_str(&std::fs::read_to_string(dir.join("kb.mars")).map_err(io)?).map_err(io)?;
        let stats = FeatureStats::from_bytes(&std::fs::read(dir.join("epoch.idf")).map_err(io)?)?;
        let ident = match std::fs::read(dir.join("identity.idf")) {
            Ok(b) => Some(FeatureStats::from_bytes(&b)?),
            Err(_) => None,
        };
        let mut e = Engine::with_epochs(kb, cfg, stats, ident);
        e.replaying = true;
        for line in std::fs::read_to_string(dir.join("meta.txt")).map_err(io)?.lines() {
            e.apply(line)?;
        }
        if let Ok(f) = File::open(dir.join("log.txt")) {
            for line in BufReader::new(f).lines() {
                let line = line.map_err(io)?;
                if !line.trim().is_empty() {
                    e.apply(&line)?;
                }
            }
        }
        e.replaying = false;
        e.attach_log(&dir.join("log.txt"))?;
        Ok(e)
    }

    fn case_arg(&self, name: &str) -> Result<CaseId, String> {
        self.kb.case_by_name(name).ok_or_else(|| format!("unknown case {name}"))
    }

    /// Apply one log/meta record (also used by interactive front ends).
    pub fn apply(&mut self, line: &str) -> Result<(), String> {
        let line = line.trim();
        let (op, rest) = line.split_once(' ').unwrap_or((line, ""));
        let rest = rest.trim();
        match op {
            "declare" => {
                self.kb.load_str(rest).map_err(io)?;
                self.log_line(format!("declare {rest}"));
            }
            "add-case" => {
                let form = sexpr::parse_one(rest).map_err(io)?;
                let items = form.as_list().ok_or("add-case needs (defcase name facts...)")?;
                let name = items.get(1).and_then(|x| x.as_atom()).ok_or("defcase needs a name")?.to_string();
                let mut facts = Vec::new();
                for f in &items[2..] {
                    match self.kb.term_from_sexp(f).map_err(io)? {
                        mars_rel::Term::Expr(e) => facts.push(e),
                        _ => return Err("facts must be expressions".into()),
                    }
                }
                self.add_case(&name, facts);
            }
            "add-fact" | "remove-fact" => {
                let (case, expr) = rest.split_once(' ').ok_or("expected <case> <expr>")?;
                let c = self.case_arg(case)?;
                let e = self.kb.parse_expr(expr.trim()).map_err(io)?;
                if op == "add-fact" {
                    self.add_fact(c, e);
                } else {
                    self.remove_fact(c, e);
                }
            }
            "remove-case" | "retire" => {
                let c = self.case_arg(rest)?;
                self.remove_case(c);
            }
            "standing" => {
                let (case, k) = rest.split_once(' ').unwrap_or((rest, "5"));
                let c = self.case_arg(case)?;
                let k: usize = k.trim().parse().map_err(io)?;
                self.add_standing_query(c, k);
            }
            "transfer-outcome" => {
                let (ok, keys) = rest.split_once(' ').unwrap_or((rest, ""));
                let keys: Vec<String> = keys.split_whitespace().map(String::from).collect();
                self.record_transfer(&keys, ok == "correct");
            }
            "transfer-total" | "transfer-counts" => {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                let num = |s: &str| s.parse::<f64>().map_err(io);
                match (op, parts.as_slice()) {
                    ("transfer-total", [h, n]) => self.transfers.set_total(num(h)?, num(n)?),
                    ("transfer-counts", [k, h, n]) => self.transfers.set_counts(k, num(h)?, num(n)?),
                    _ => return Err(format!("bad record {line}")),
                }
            }
            other => return Err(format!("unknown record {other}")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
(defpredicate cause :arity 2 :kind relation)
(defcase solar (attracts sun planet) (revolve-around planet sun) (cause (attracts sun planet) (revolve-around planet sun)))
(defcase other (attracts x y) (repels y z))
(defcase atom (attracts nucleus electron) (revolve-around electron nucleus))
"#;

    #[test]
    fn checkpoint_log_and_reopen_reproduce_state() {
        let dir = std::env::temp_dir().join(format!("mars-store-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut kb = Kb::new();
        kb.load_str(SRC).unwrap();
        let mut e = Engine::new(kb, EngineConfig::default());
        let atom = e.kb.case_by_name("atom").unwrap();
        e.add_standing_query(atom, 2);
        e.checkpoint(&dir).unwrap();
        // Logged operations after the snapshot.
        e.apply("add-case (defcase star (attracts star moon) (revolve-around moon star) (cause (attracts star moon) (revolve-around moon star)))").unwrap();
        e.apply("remove-fact solar (cause (attracts sun planet) (revolve-around planet sun))").unwrap();
        e.apply("retire other").unwrap();
        let before_top = e.standing(0).top().to_vec();
        let before_inf = e.inferences(0);
        drop(e);
        let mut r = Engine::open(&dir, EngineConfig::default()).unwrap();
        assert_eq!(r.standing(0).top(), &before_top[..]);
        assert_eq!(r.inferences(0), before_inf);
        assert!(!r.is_alive(r.kb.case_by_name("other").unwrap()));
        // Checkpoint again and reopen: still identical.
        r.checkpoint(&dir).unwrap();
        drop(r);
        let r2 = Engine::open(&dir, EngineConfig::default()).unwrap();
        assert_eq!(r2.standing(0).top(), &before_top[..]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
