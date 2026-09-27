//! Python bindings for MARS.
//!
//! ```python
//! import mars
//! e = mars.Engine.from_files(["data/examples/classic.mars"])
//! hits, z = e.query("rutherford-atom", k=3)   # [(case, fused score)], local-null z
//! m = e.map("solar-system", "rutherford-atom") # score, entity/expression matches, inferences
//! sq = e.watch("rutherford-atom", k=3)         # standing query, maintained incrementally
//! e.add_case("(defcase star (attracts star moon) (revolve-around moon star))")
//! e.top(sq); e.infer(sq, min_support=1); e.explain(sq, text)
//! e.checkpoint("store/"); e2 = mars.Engine.open("store/")
//! ```
//!
//! Cases use the `.mars` s-expression format (`mars_rel::load`). Mutations go
//! through the same records as the persistence log and `mars serve`, so a
//! checkpointed engine replays them exactly.

use mars_engine::{Engine, EngineConfig, SIGNIFICANT_Z};
use mars_map::{Grounding, MapConfig, Mapper};
use mars_rel::{CaseId, Kb, Term};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::Path;

/// Ranked analogues `[(case, score)]`.
type Hits = Vec<(String, f64)>;

fn err(e: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(e.to_string())
}

/// A MARS engine: incremental analogical memory with standing queries.
#[pyclass(unsendable, name = "Engine")]
struct PyEngine {
    e: Engine,
}

impl PyEngine {
    fn case(&self, name: &str) -> PyResult<CaseId> {
        self.e.kb.case_by_name(name).ok_or_else(|| PyKeyError::new_err(format!("unknown case {name}")))
    }

    fn name(&self, c: CaseId) -> String {
        self.e.kb.name(self.e.kb.case(c).name).to_string()
    }

    fn check_sq(&self, sq: usize) -> PyResult<()> {
        if sq >= self.e.n_standing() {
            return Err(PyKeyError::new_err(format!("no standing query {sq}")));
        }
        Ok(())
    }
}

#[pymethods]
impl PyEngine {
    /// Build an engine from `.mars` source text (declarations and cases).
    #[new]
    #[pyo3(signature = (source = ""))]
    fn new(source: &str) -> PyResult<Self> {
        let mut kb = Kb::new();
        kb.load_str(source).map_err(err)?;
        Ok(PyEngine { e: Engine::new(kb, EngineConfig::default()) })
    }

    /// Build an engine from `.mars` files.
    #[staticmethod]
    fn from_files(paths: Vec<String>) -> PyResult<Self> {
        let mut kb = Kb::new();
        for p in &paths {
            let src = std::fs::read_to_string(p).map_err(|e| err(format!("{p}: {e}")))?;
            kb.load_str(&src).map_err(|e| err(format!("{p}: {e}")))?;
        }
        Ok(PyEngine { e: Engine::new(kb, EngineConfig::default()) })
    }

    /// Open a checkpointed store directory (snapshot + frozen IDF epoch + log).
    #[staticmethod]
    fn open(dir: &str) -> PyResult<Self> {
        Ok(PyEngine { e: Engine::open(Path::new(dir), EngineConfig::default()).map_err(err)? })
    }

    /// Write a snapshot to `dir` and log subsequent mutations there.
    fn checkpoint(&mut self, dir: &str) -> PyResult<()> {
        self.e.checkpoint(Path::new(dir)).map_err(err)
    }

    /// Apply one record: `declare ...`, `add-case (defcase ...)`, `add-fact CASE (expr)`,
    /// `remove-fact CASE (expr)`, `remove-case CASE`, `standing CASE K`.
    fn apply(&mut self, record: &str) -> PyResult<()> {
        self.e.apply(record).map_err(err)
    }

    /// Add a case given as `(defcase NAME fact ...)`.
    fn add_case(&mut self, defcase: &str) -> PyResult<()> {
        self.e.apply(&format!("add-case {defcase}")).map_err(err)
    }

    /// Add or remove one fact of a case, e.g. `add_fact("solar", "(attracts sun planet)")`.
    fn add_fact(&mut self, case: &str, fact: &str) -> PyResult<()> {
        self.e.apply(&format!("add-fact {case} {fact}")).map_err(err)
    }

    fn remove_fact(&mut self, case: &str, fact: &str) -> PyResult<()> {
        self.e.apply(&format!("remove-fact {case} {fact}")).map_err(err)
    }

    /// Retire a case (it is no longer retrieved).
    fn remove_case(&mut self, case: &str) -> PyResult<()> {
        self.e.apply(&format!("remove-case {case}")).map_err(err)
    }

    /// Names of the live cases.
    fn cases(&self) -> Vec<String> {
        (0..self.e.kb.n_cases()).map(|i| CaseId(i as u32)).filter(|&c| self.e.is_alive(c)).map(|c| self.name(c)).collect()
    }

    /// The case as `.mars` text.
    fn render(&self, case: &str) -> PyResult<String> {
        Ok(self.e.kb.render_case(self.case(case)?))
    }

    /// Analogues of a case: ([(name, fused score)], significance z of the top-1 or None).
    /// Accept the top-1 as a real analogue when z >= mars.SIGNIFICANT_Z (E16).
    #[pyo3(signature = (case, k = 5))]
    fn query(&mut self, case: &str, k: usize) -> PyResult<(Hits, Option<f64>)> {
        let q = self.case(case)?;
        let (hits, z) = self.e.query_significance(q, k);
        Ok((hits.into_iter().map(|(c, s)| (self.name(c), s)).collect(), z))
    }

    /// Normalized structural (FAC) score of mapping `base` onto `target`.
    fn fac(&mut self, base: &str, target: &str) -> PyResult<f64> {
        let (b, t) = (self.case(base)?, self.case(target)?);
        Ok(self.e.fac(t, b))
    }

    /// Structural mapping base → target: dict with `score`, `entities` [(base, target)],
    /// `matches` [(base fact, target fact)], `inferences` [(fact, support, has_skolem)]
    /// (structurally grounded candidate inferences) and `differences`.
    fn map<'py>(&self, py: Python<'py>, base: &str, target: &str) -> PyResult<Bound<'py, PyDict>> {
        let (b, t) = (self.case(base)?, self.case(target)?);
        let kb = &self.e.kb;
        let mp = Mapper::new(kb, MapConfig::default());
        let d = PyDict::new(py);
        let Some(m) = mp.best(b, t) else {
            d.set_item("score", 0.0)?;
            return Ok(d);
        };
        let ents: Vec<(String, String)> = m.correspondences.iter().filter_map(|(x, y)| if let (Term::Ent(x), Term::Ent(y)) = (x, y) { Some((kb.name(*x).to_string(), kb.name(*y).to_string())) } else { None }).collect();
        let base_facts = &kb.case(b).facts;
        let matches: Vec<(String, String)> = m.correspondences.iter().filter_map(|(x, y)| if let (Term::Expr(xe), Term::Expr(_)) = (x, y) { base_facts.contains(xe).then(|| (kb.render_term(*x), kb.render_term(*y))) } else { None }).collect();
        let infs: Vec<(String, f32, bool)> = m.inferences.iter().filter(|i| i.grounding == Grounding::Structural).map(|i| (mp.render_proj(&i.projected), i.support, i.has_skolem)).collect();
        let diffs: Vec<(String, String)> = m.alignable_differences.iter().map(|(x, y)| (kb.render_expr(*x), kb.render_expr(*y))).collect();
        d.set_item("score", m.score)?;
        d.set_item("entities", ents)?;
        d.set_item("matches", matches)?;
        d.set_item("inferences", infs)?;
        d.set_item("differences", diffs)?;
        Ok(d)
    }

    /// Register a standing query: its top-k analogues and inferences are maintained
    /// incrementally as the memory changes. Returns its id.
    #[pyo3(signature = (case, k = 5))]
    fn watch(&mut self, case: &str, k: usize) -> PyResult<usize> {
        self.e.apply(&format!("standing {case} {k}")).map_err(err)?;
        Ok(self.e.n_standing() - 1)
    }

    /// Current analogues of a standing query: [(name, fused score)].
    fn top(&self, sq: usize) -> PyResult<Hits> {
        self.check_sq(sq)?;
        Ok(self.e.standing(sq).top().iter().map(|&(c, s)| (self.name(c), s)).collect())
    }

    /// Believed candidate inferences of a standing query with their support
    /// (number of corroborating analogues; the calibrated confidence of E11).
    #[pyo3(signature = (sq, min_support = 1))]
    fn infer(&self, sq: usize, min_support: usize) -> PyResult<Vec<(String, usize)>> {
        self.check_sq(sq)?;
        Ok(self.e.corroborated(sq, min_support))
    }

    /// Provenance (JTMS justifications) of one inference of a standing query.
    fn explain(&self, sq: usize, inference: &str) -> PyResult<Option<String>> {
        self.check_sq(sq)?;
        Ok(self.e.explain(sq, inference))
    }

    /// Drain change events (standing-query results and inferences) as strings.
    fn events(&mut self) -> Vec<String> {
        self.e.drain_events().into_iter().map(|ev| format!("{ev:?}")).collect()
    }

    fn __len__(&self) -> usize {
        self.e.n_live()
    }

    fn __repr__(&self) -> String {
        format!("<mars.Engine: {} live cases, {} standing queries>", self.e.n_live(), self.e.n_standing())
    }
}

#[pymodule]
fn mars(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngine>()?;
    m.add("SIGNIFICANT_Z", SIGNIFICANT_Z)?;
    m.add("__doc__", "MARS: structural analogical memory (fingerprint retrieval, structure mapping, incremental truth maintenance).")?;
    Ok(())
}
