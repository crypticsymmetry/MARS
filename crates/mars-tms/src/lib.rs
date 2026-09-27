//! `mars-tms`: a justification-based truth maintenance system (Doyle 1979).
//!
//! Nodes are IN or OUT. A node is IN iff it is an enabled premise or has a
//! justification whose antecedents are all IN. Each IN derived node records
//! one *well-founded* supporting justification, so circular justifications
//! cannot keep each other alive: on retraction, every node whose support
//! chain passed through the retracted node is marked OUT, then re-supported
//! only from nodes that are still IN.
//!
//! `touched` counts node label evaluations, the work measure used by E6.

use smallvec::SmallVec;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct JustId(pub u32);

#[derive(Clone, Debug, Default)]
struct Node {
    is_in: bool,
    premise: bool,
    /// Justifications concluding this node.
    justs: Vec<JustId>,
    /// Justifications in which this node is an antecedent.
    consumers: Vec<JustId>,
    /// The well-founded justification currently supporting this node.
    support: Option<JustId>,
}

#[derive(Clone, Debug)]
struct Just {
    consequent: NodeId,
    antecedents: SmallVec<[NodeId; 4]>,
    active: bool,
}

#[derive(Default, Clone, Debug)]
pub struct Jtms {
    nodes: Vec<Node>,
    justs: Vec<Just>,
    /// Work counter: node label evaluations.
    pub touched: u64,
}

impl Jtms {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self) -> NodeId {
        self.nodes.push(Node::default());
        NodeId(self.nodes.len() as u32 - 1)
    }

    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    pub fn is_in(&self, n: NodeId) -> bool {
        self.nodes[n.0 as usize].is_in
    }

    pub fn is_premise(&self, n: NodeId) -> bool {
        self.nodes[n.0 as usize].premise
    }

    /// The justification currently supporting `n` (None for premises / OUT nodes).
    pub fn support(&self, n: NodeId) -> Option<(NodeId, Vec<NodeId>)> {
        self.nodes[n.0 as usize].support.map(|j| {
            let j = &self.justs[j.0 as usize];
            (j.consequent, j.antecedents.to_vec())
        })
    }

    /// Number of currently valid justifications of `n` (its corroboration).
    pub fn support_count(&self, n: NodeId) -> usize {
        self.nodes[n.0 as usize].justs.iter().filter(|&&j| self.just_holds(j)).count()
    }

    /// Enable a node as a premise (assumption believed without justification).
    pub fn assume(&mut self, n: NodeId) {
        let node = &mut self.nodes[n.0 as usize];
        if node.premise {
            return;
        }
        node.premise = true;
        if !node.is_in {
            node.is_in = true;
            node.support = None;
            self.touched += 1;
            self.propagate_in(vec![n]);
        }
    }

    /// Retract a premise. Returns nodes that went OUT.
    pub fn retract(&mut self, n: NodeId) -> Vec<NodeId> {
        let node = &mut self.nodes[n.0 as usize];
        if !node.premise {
            return Vec::new();
        }
        node.premise = false;
        self.relabel_from(n)
    }

    /// Add a justification `antecedents ⊢ consequent`.
    pub fn justify(&mut self, consequent: NodeId, antecedents: &[NodeId]) -> JustId {
        let j = JustId(self.justs.len() as u32);
        self.justs.push(Just { consequent, antecedents: antecedents.iter().copied().collect(), active: true });
        self.nodes[consequent.0 as usize].justs.push(j);
        for &a in antecedents {
            self.nodes[a.0 as usize].consumers.push(j);
        }
        if !self.is_in(consequent) && antecedents.iter().all(|&a| self.is_in(a)) {
            let c = &mut self.nodes[consequent.0 as usize];
            c.is_in = true;
            c.support = Some(j);
            self.touched += 1;
            self.propagate_in(vec![consequent]);
        }
        j
    }

    /// Deactivate a justification. Returns nodes that went OUT.
    pub fn unjustify(&mut self, j: JustId) -> Vec<NodeId> {
        let just = &mut self.justs[j.0 as usize];
        if !just.active {
            return Vec::new();
        }
        just.active = false;
        let c = just.consequent;
        if self.nodes[c.0 as usize].support == Some(j) {
            self.relabel_from(c)
        } else {
            Vec::new()
        }
    }

    pub fn just_holds(&self, j: JustId) -> bool {
        let just = &self.justs[j.0 as usize];
        just.active && just.antecedents.iter().all(|&a| self.nodes[a.0 as usize].is_in)
    }

    /// Forward propagation of newly IN nodes.
    fn propagate_in(&mut self, mut queue: Vec<NodeId>) {
        while let Some(n) = queue.pop() {
            let consumers = self.nodes[n.0 as usize].consumers.clone();
            for j in consumers {
                let c = self.justs[j.0 as usize].consequent;
                self.touched += 1;
                if !self.nodes[c.0 as usize].is_in && self.just_holds(j) {
                    let node = &mut self.nodes[c.0 as usize];
                    node.is_in = true;
                    node.support = Some(j);
                    queue.push(c);
                }
            }
        }
    }

    /// `start` lost its support (or premise status): mark OUT everything whose
    /// support chain depends on it, then re-support what still can be.
    fn relabel_from(&mut self, start: NodeId) -> Vec<NodeId> {
        // 1. Collect the support-dependent region.
        let mut region = vec![start];
        let mut in_region = vec![false; self.nodes.len()];
        in_region[start.0 as usize] = true;
        let mut i = 0;
        while i < region.len() {
            let n = region[i];
            i += 1;
            let consumers = self.nodes[n.0 as usize].consumers.clone();
            for j in consumers {
                let c = self.justs[j.0 as usize].consequent;
                if !in_region[c.0 as usize] && self.nodes[c.0 as usize].support == Some(j) {
                    in_region[c.0 as usize] = true;
                    region.push(c);
                }
            }
        }
        // 2. Mark the region OUT (premises stay IN).
        for &n in &region {
            self.touched += 1;
            let node = &mut self.nodes[n.0 as usize];
            if !node.premise {
                node.is_in = false;
                node.support = None;
            }
        }
        // 3. Re-support to a fixed point using only IN antecedents (well-founded).
        let mut changed = true;
        while changed {
            changed = false;
            for &n in &region {
                if self.nodes[n.0 as usize].is_in {
                    continue;
                }
                self.touched += 1;
                let js = self.nodes[n.0 as usize].justs.clone();
                if let Some(&j) = js.iter().find(|&&j| self.just_holds(j)) {
                    let node = &mut self.nodes[n.0 as usize];
                    node.is_in = true;
                    node.support = Some(j);
                    changed = true;
                }
            }
        }
        // Newly re-supported nodes may enable consumers outside the region.
        let back_in: Vec<NodeId> = region.iter().copied().filter(|&n| self.nodes[n.0 as usize].is_in).collect();
        self.propagate_in(back_in);
        region.into_iter().filter(|&n| !self.nodes[n.0 as usize].is_in).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_retraction_and_alternative_support() {
        let mut t = Jtms::new();
        let (a, b, c, d) = (t.add_node(), t.add_node(), t.add_node(), t.add_node());
        t.assume(a);
        t.assume(b);
        t.justify(c, &[a]); // a ⊢ c
        t.justify(c, &[b]); // b ⊢ c (alternative)
        t.justify(d, &[c]); // c ⊢ d
        assert!(t.is_in(c) && t.is_in(d));
        let out = t.retract(a);
        assert_eq!(out, vec![a], "c still supported by b");
        assert!(t.is_in(c) && t.is_in(d));
        let out = t.retract(b);
        assert!(!t.is_in(c) && !t.is_in(d));
        assert_eq!(out, vec![b, c, d]);
        t.assume(b);
        assert!(t.is_in(d), "re-assuming restores consequences");
    }

    #[test]
    fn circular_support_is_not_well_founded() {
        let mut t = Jtms::new();
        let (p, x, y) = (t.add_node(), t.add_node(), t.add_node());
        t.assume(p);
        t.justify(x, &[p]);
        t.justify(y, &[x]);
        t.justify(x, &[y]); // cycle x <-> y
        assert!(t.is_in(x) && t.is_in(y));
        t.retract(p);
        assert!(!t.is_in(x) && !t.is_in(y), "cycle must not keep itself alive");
    }

    #[test]
    fn unjustify_and_conjunction() {
        let mut t = Jtms::new();
        let (a, b, c) = (t.add_node(), t.add_node(), t.add_node());
        t.assume(a);
        let j = t.justify(c, &[a, b]);
        assert!(!t.is_in(c), "b not IN yet");
        t.assume(b);
        assert!(t.is_in(c));
        t.unjustify(j);
        assert!(!t.is_in(c));
    }
}
