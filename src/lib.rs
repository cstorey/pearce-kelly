use std::{collections::HashSet, fmt, hash::Hash};

use petgraph::{
    graph::NodeIndex,
    stable_graph::{EdgeReference, StableGraph},
    visit::IntoEdgeReferences,
    Direction,
};

mod error;

use crate::error::Result;

pub use crate::error::TopologicalOrderingError;

// With reference to abseil, licensed under Apache2:
// [graphcycles.cc](https://github.com/abseil/abseil-cpp/blob/9c6a50fdd80bb39fabd95faeda84f04062685ff3/absl/synchronization/internal/graphcycles.cc)

type InnerGraph<N, E> = StableGraph<N, E>;

// TODO
// Implement petgraph's Build / GraphBase traits?
#[derive(Debug, Clone, Default)]
pub struct ToplogicalOrdering<N, E> {
    items: Vec<NodeIndex>,
    // Also needs a reference to the graph.
    inner: InnerGraph<N, E>,
}

// The state for an insertion of node x → y
#[derive(Debug, Clone)]
struct InsertState {
    lb: usize,
    ub: usize,
    visited: HashSet<NodeIndex>,
    // Pearce's C++ code calls this `reachable`
    // All nodes in the transitive closure reachable from x
    delta_f_xy: Vec<NodeIndex>,
    // Same, but `reaching`
    // All nodes in the transitive closure that reach y
    delta_b_xy: Vec<NodeIndex>,
}

impl<N, E> ToplogicalOrdering<N, E>
where
    N: Eq + Clone + Hash + fmt::Debug,
{
    pub fn add_edge(&mut self, x: NodeIndex, y: NodeIndex, weight: E) -> Result<()> {
        let ub = self.ensure_rank(x);
        let lb = self.ensure_rank(y);
        println!("x: {:?}; ub:{:?}; y: {:?}; lb: {:?}", x, ub, y, lb);
        // Assuming that the insertion would result in the graph still being acyclic,
        // and our order needs rejigging, then:
        if lb < ub {
            println!(
                "Affected range: {:?}",
                self.items[lb..=ub]
                    .iter()
                    .cloned()
                    .map(|x| self.inner.node_weight(x).expect("node"))
                    .collect::<Vec<_>>()
            );
            let mut state = InsertState {
                lb,
                ub,
                visited: HashSet::new(),
                delta_f_xy: Vec::new(),
                delta_b_xy: Vec::new(),
            };
            // Find the set of nodes that are reachable from `y` (dst)
            self.dfs_f(&mut state, y)?;
            // Find the set of nodes that reach to `x` (src).
            self.dfs_b(&mut state, x)?;
            eprintln!("state: {:?}", state);

            // And I think ths kills the cra^W^W^WEffectivly swaps the order,
            // So items reachable from `y` now all come after thse that reach `x`.
            self.reorder(&mut state)?;
        }

        println!("Post insert {:?} → {:?}: {:?}", x, y, self.items);

        self.inner.add_edge(x, y, weight);

        Ok(())
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = &'_ NodeIndex> + '_ {
        self.items.iter()
    }
    fn dfs_f(&self, state: &mut InsertState, n: NodeIndex) -> Result<()> {
        state.visited.insert(n);
        state.delta_f_xy.push(n);
        println!("DFS-f: visit: {:?}", self.inner.node_weight(n));

        for w in self.inner.neighbors_directed(n, Direction::Outgoing) {
            let ordw = self.rank(w)?;
            if ordw == state.ub {
                return Err(TopologicalOrderingError::CycleDetected);
            }
            if !state.visited.contains(&w) && ordw < state.ub {
                println!(
                    "DFS-f: {:?} → {:?}",
                    self.inner.node_weight(n),
                    self.inner.node_weight(w)
                );
                self.dfs_f(state, w)?
            }
        }

        Ok(())
    }

    fn dfs_b(&self, state: &mut InsertState, n: NodeIndex) -> Result<()> {
        state.visited.insert(n);
        state.delta_b_xy.push(n);
        println!("DFS-b: visit: {:?}", self.inner.node_weight(n));

        for w in self.inner.neighbors_directed(n, Direction::Incoming) {
            let ordw = self.rank(w)?;
            if ordw == state.ub {
                // return Err(TopologicalOrderingError::CycleDetected);
            }
            if !state.visited.contains(&w) && state.lb < ordw {
                println!(
                    "DFS-b: {:?} → {:?}",
                    self.inner.node_weight(n),
                    self.inner.node_weight(w)
                );
                self.dfs_b(state, w)?
            }
        }
        Ok(())
    }

    fn reorder(&mut self, state: &mut InsertState) -> Result<()> {
        self.sort_by_current_rank(&mut state.delta_b_xy);
        self.sort_by_current_rank(&mut state.delta_f_xy);
        // So now, both delta_b_xy and delta_f_xy are topoligcally ordered.

        // In some sense, "all" we need to do is glue them together.

        // L in the paper
        let mut l = Vec::<NodeIndex>::new();
        // The target ranks for each element, I think?
        let mut out_delta_b_xy = Vec::<usize>::with_capacity(state.delta_b_xy.len());
        let mut out_delta_f_xy = Vec::<usize>::with_capacity(state.delta_f_xy.len());
        for (i, w) in state.delta_b_xy.iter().cloned().enumerate() {
            // Wat?
            // This is used by the merge step, but who knows what it's mean to
            // mean, given ord is effectively NodeIndex -> usize
            // δBxy [i] = ord[w];
            let rank_w = self.rank(w)?;
            eprintln!("δBxy[{}] = ord[{:?}] ({:?})", i, w, rank_w,);
            out_delta_b_xy.push(rank_w);
            state.visited.remove(&w);
            l.push(w);
        }
        for (i, w) in state.delta_f_xy.iter().cloned().enumerate() {
            // Wat?
            // This is used by the merge step, but who knows what it's mean to
            // mean, given ord is effectively NodeIndex -> usize
            // δFxy[i] = ord[w];
            let rank_w = self.rank(w)?;
            eprintln!("δFxy[{}] = ord[{:?}] ({:?})", i, w, rank_w,);
            out_delta_f_xy.push(rank_w);
            state.visited.remove(&w);
            l.push(w);
        }

        eprintln!("δBxy: {:?}", out_delta_b_xy);
        eprintln!("δFxy: {:?}", out_delta_f_xy);

        // R in the paper.
        // The "pool of available indexes"
        let mut r = Vec::<usize>::new();
        {
            let mut delta_b_xy_it = out_delta_b_xy.iter().copied().peekable();
            let mut delta_f_xy_it = out_delta_f_xy.iter().copied().peekable();
            loop {
                match (delta_b_xy_it.peek(), delta_f_xy_it.peek()) {
                    (Some(f_it), Some(b_it)) if f_it < b_it => {
                        r.push(delta_b_xy_it.next().unwrap())
                    }
                    (Some(_f_it), Some(_b_it)) => r.push(delta_f_xy_it.next().unwrap()),
                    (Some(_f_it), None) => r.push(delta_b_xy_it.next().unwrap()),
                    (None, Some(_b_it)) => r.push(delta_f_xy_it.next().unwrap()),
                    (None, None) => break,
                }
            }
        }

        eprintln!(
            "Merged: {:?}",
            r,
            // merged.iter().filter_map(|ix| self.inner.node_weight(*ix).map(|v| (ix, v))).collect::<Vec<_>>()
        );
        eprintln!(
            "L: {:?}",
            l.iter()
                .filter_map(|ix| self.inner.node_weight(*ix).map(|v| (ix, v)))
                .collect::<Vec<_>>()
        );

        eprintln!("TODO: assign");
        // for i = 0 to |L|−1
        for (l_i, r_i) in l.into_iter().zip(r.into_iter()) {
            self.items[r_i] = l_i;
        }

        Ok(())
    }

    fn sort_by_current_rank(&mut self, delta: &mut Vec<NodeIndex>) {
        delta.sort_by_key(|x| self.rank(*x).expect("rank present"));
    }

    fn ensure_rank(&mut self, item: NodeIndex) -> usize {
        if let Some(ix) = self
            .items
            .iter()
            .enumerate()
            .find(|(_, v)| *v == &item)
            .map(|(ix, _)| ix)
        {
            ix
        } else {
            let ix = self.items.len();
            self.items.push(item);
            ix
        }
    }

    fn rank(&self, item: NodeIndex) -> Result<usize> {
        if let Some(ix) = self
            .items
            .iter()
            .enumerate()
            .find(|(_, v)| *v == &item)
            .map(|(ix, _)| ix)
        {
            Ok(ix)
        } else {
            Err(TopologicalOrderingError::NoSuchItem(item))
        }
    }

    pub fn add_node(&mut self, weight: N) -> NodeIndex {
        let res: NodeIndex = self.inner.add_node(weight);
        res
    }

    pub fn node_weight(&self, ix: NodeIndex) -> Option<&N> {
        self.inner.node_weight(ix)
    }

    pub fn edge_references(&self) -> impl Iterator<Item = EdgeReference<E, u32>> {
        self.inner.edge_references()
    }
}
