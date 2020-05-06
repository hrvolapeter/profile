#![deny(warnings)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::default_trait_access,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::use_self,
    clippy::single_match_else,
    clippy::wildcard_imports,
    clippy::new_without_default,
    clippy::cast_sign_loss
)]

mod bfs;
mod ford_fulkerson;
mod minimum_cost_flow;

use bfs::BFS;
pub use ford_fulkerson::FordFulkerson;
pub use minimum_cost_flow::MinimumCostFlow;
use std::fmt::Debug;

pub trait Graphable {
    fn name_label(&self) -> String;
}

impl Graphable for &str {
    fn name_label(&self) -> String {
        self.to_string()
    }
}

impl Graphable for () {
    fn name_label(&self) -> String {
        "()".to_string()
    }
}

impl Graphable for u32 {
    fn name_label(&self) -> String {
        format!("{}", self)
    }
}

#[derive(Clone, Debug)]
pub struct Graph<T: Debug> {
    nodes: Vec<NodeData<T>>,
    edges: Vec<EdgeData>,
    pub source: NodeIndex,
    pub sink: NodeIndex,
}

// NODE
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeIndex(usize);

#[derive(Clone, Debug)]
pub struct NodeData<T: Debug> {
    first_outgoing_edge: Option<EdgeIndex>,
    pub inner: Node<T>,
    index: NodeIndex,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Node<T: Debug> {
    Sink,
    Source,
    Node(T),
}

// EDGE

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct EdgeIndex(usize);

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug, Hash)]
pub struct Cost(pub i64);

impl Cost {
    const MAX: Cost = Self(i64::MAX);
}

impl std::ops::Add for Cost {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.0.checked_add(rhs.0).map_or(Cost::MAX, Self)
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug, Hash)]
pub struct Capacity(pub i64);

impl Capacity {
    const MAX: Capacity = Self(i64::MAX);
}

impl std::ops::SubAssign for Capacity {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::AddAssign for Capacity {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug, Hash)]
struct Flow(pub i64);

impl std::ops::SubAssign for Flow {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::AddAssign for Flow {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct EdgeData {
    index: EdgeIndex,
    cost: Cost,
    capacity: Capacity,
    flow: Flow,
    target: NodeIndex,
    source: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>,
}

impl EdgeData {
    fn residual_capacity(&self) -> Capacity {
        Capacity(self.capacity.0 - self.flow.0)
    }
}

#[derive(Debug)]
struct PathInner(Vec<EdgeIndex>);

/// Represents flow in a solution to the minimum cost maximum flow problem.
#[derive(Clone, Debug)]
pub struct Edge<T: Clone + Debug> {
    pub source: Node<T>,
    pub target: Node<T>,
    pub flow: u64,
    pub capacity: u64,
    pub cost: u64,
}

/// Represents a path from the source to the sink in a solution to the minimum cost maximum flow problem.
#[derive(Debug)]
pub struct Path<T: Clone + Debug> {
    pub edges: Vec<Edge<T>>,
}

impl<T: Debug> Graph<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: vec![
                NodeData { first_outgoing_edge: None, inner: Node::Source, index: NodeIndex(0) },
                NodeData { first_outgoing_edge: None, inner: Node::Sink, index: NodeIndex(1) },
            ],
            edges: vec![],
            source: NodeIndex(0),
            sink: NodeIndex(1),
        }
    }

    #[must_use]
    pub fn add_node(&mut self, inner: T) -> NodeIndex {
        let index = NodeIndex(self.nodes.len());
        self.nodes.push(NodeData { index, first_outgoing_edge: None, inner: Node::Node(inner) });
        index
    }

    pub fn add_edge(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        capacity: Capacity,
        cost: Cost,
    ) {
        self.add_edge_with_flow(source, target, capacity, cost, Flow(0))
    }

    fn add_edge_with_flow(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        capacity: Capacity,
        cost: Cost,
        flow: Flow,
    ) {
        let edge_index = EdgeIndex(self.edges.len());
        let node_data = &mut self.nodes[source.0];
        self.edges.push(EdgeData {
            source,
            target,
            next_outgoing_edge: node_data.first_outgoing_edge,
            cost,
            flow,
            capacity,
            index: edge_index,
        });
        node_data.first_outgoing_edge = Some(edge_index);
    }

    fn residual_graph(&self) -> (Graph<()>, Vec<Result<EdgeIndex, EdgeIndex>>) {
        let mut res = Graph::new();
        for _ in &self.nodes {
            let _ = res.add_node(());
        }
        let mut mapping = vec![];
        for edge in &self.edges {
            if edge.capacity.0 != edge.flow.0 {
                mapping.push(Ok(edge.index));
                res.add_edge(edge.source, edge.target, edge.residual_capacity(), edge.cost);
            }
            mapping.push(Err(edge.index));
            res.add_edge(edge.target, edge.source, Capacity(edge.flow.0), Cost(-edge.cost.0));
        }
        (res, mapping)
    }

    #[allow(dead_code)]
    fn successors(&self, source: NodeIndex) -> Successors<T> {
        let first_outgoing_edge = self.nodes[source.0].first_outgoing_edge;
        Successors { graph: self, current_edge_index: first_outgoing_edge }
    }

    fn edges(&self, source: NodeIndex) -> Edges<T> {
        let first_outgoing_edge = self.nodes[source.0].first_outgoing_edge;
        Edges { graph: self, current_edge_index: first_outgoing_edge }
    }

    fn paths_inner(&self, node: NodeIndex) -> Vec<PathInner> {
        let edges = self.edges(node).filter(|x| x.flow.0 > 0 && x.capacity.0 > 0);
        let mut res = vec![];
        for edge in edges {
            let mut paths = self.paths_inner(edge.target);
            for path in &mut paths {
                path.0.push(edge.index);
            }
            if paths.is_empty() {
                paths.push(PathInner(vec![edge.index]));
            }

            res.append(&mut paths);
        }
        res
    }
}

impl<T: Clone + Debug> Graph<T> {
    #[must_use]
    pub fn paths(&self) -> Vec<Path<T>> {
        self.paths_inner(self.source)
            .into_iter()
            .map(|mut x| {
                x.0.reverse();
                let edges =
                    x.0.into_iter()
                        .map(|x| {
                            let edge = &self.edges[x.0];
                            Edge {
                                source: self.nodes[edge.source.0].inner.clone(),
                                target: self.nodes[edge.target.0].inner.clone(),
                                capacity: edge.capacity.0 as u64,
                                flow: edge.flow.0 as u64,
                                cost: edge.cost.0 as u64,
                            }
                        })
                        .collect();
                Path { edges }
            })
            .collect()
    }

    #[must_use]
    pub fn all_edges(&self) -> Vec<Edge<T>> {
        self.edges
            .iter()
            .map(|edge| Edge {
                source: self.nodes[edge.source.0].inner.clone(),
                target: self.nodes[edge.target.0].inner.clone(),
                capacity: edge.capacity.0 as u64,
                flow: edge.flow.0 as u64,
                cost: edge.cost.0 as u64,
            })
            .collect()
    }
}

impl<T: Clone + Debug + Graphable> Graph<T> {
    #[must_use]
    pub fn graphviz(&self) -> String {
        let inner: String = self
            .edges
            .iter()
            .map(|e| {
                let cost =
                    if e.cost == Cost::MAX { "inf".to_string() } else { format!("{}", e.cost.0) };
                let source = match &self.nodes[e.source.0].inner {
                    Node::Node(n) => n.name_label(),
                    _ => format!("{}", e.source.0),
                };
                let target = match &self.nodes[e.target.0].inner {
                    Node::Node(n) => n.name_label(),
                    _ => format!("{}", e.target.0),
                };
                format!(
                    r#""{}" -> "{}" [label="{}/{};{}"];
"#,
                    source, target, e.flow.0, e.capacity.0, cost
                )
            })
            .collect();
        format!(
            r#"digraph g {{
{}
}}"#,
            inner
        )
    }
}

struct Successors<'graph, T: Debug> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, T: Debug> Iterator for Successors<'graph, T> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num.0];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge.target)
            }
        }
    }
}

struct Edges<'graph, T: Debug> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, T: Debug> Iterator for Edges<'graph, T> {
    type Item = &'graph EdgeData;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num.0];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_graph() {
        let _ = Graph::<i8>::new();
    }

    #[test]
    fn simple_graph() {
        let mut graph = Graph::new();

        let a = graph.add_node(1);
        let b = graph.add_node(2);
        let c = graph.add_node(3);
        graph.add_edge(a, b, Capacity(1), Cost(1));
        graph.add_edge(a, c, Capacity(2), Cost(2));
        graph.add_edge(b, c, Capacity(3), Cost(3));
        let suc = graph.successors(a).collect::<Vec<_>>();
        assert!(suc.len() == 2);
        assert!(suc == vec![c, b]);

        let suc = graph.successors(b).collect::<Vec<_>>();
        assert!(suc.len() == 1);
        assert!(suc == vec![c]);
    }

    #[test]
    fn complex_graph() {
        let mut graph = Graph::new();

        let a = graph.add_node(1);
        let b = graph.add_node(2);
        let c = graph.add_node(3);
        let d = graph.add_node(4);
        let f = graph.add_node(5);
        let g = graph.add_node(6);

        graph.add_edge(a, b, Capacity(1), Cost(1));
        graph.add_edge(a, c, Capacity(1), Cost(1));
        graph.add_edge(b, c, Capacity(1), Cost(1));
        graph.add_edge(c, d, Capacity(1), Cost(1));
        graph.add_edge(d, f, Capacity(1), Cost(1));
        graph.add_edge(f, g, Capacity(1), Cost(1));
        graph.add_edge(g, a, Capacity(1), Cost(1));
        graph.add_edge(d, g, Capacity(1), Cost(1));
        graph.add_edge(a, g, Capacity(1), Cost(1));

        let suc = graph.successors(a).collect::<Vec<_>>();
        assert!(suc.len() == 3);
        assert!(suc == vec![g, c, b]);

        let suc = graph.successors(b).collect::<Vec<_>>();
        assert!(suc.len() == 1);
        assert!(suc == vec![c]);

        let suc = graph.successors(d).collect::<Vec<_>>();
        assert!(suc.len() == 2);
        assert!(suc == vec![g, f]);

        let suc = graph.successors(g).collect::<Vec<_>>();
        assert!(suc.len() == 1);
        assert!(suc == vec![a]);
    }

    #[test]
    fn residual_graph() {
        let mut g = Graph::new();
        let a = g.add_node(2);
        let b = g.add_node(3);

        g.add_edge_with_flow(g.source, a, Capacity(2), Cost(1), Flow(2));
        g.add_edge_with_flow(g.source, b, Capacity(4), Cost(1), Flow(4));
        g.add_edge_with_flow(a, b, Capacity(3), Cost(1), Flow(1));
        g.add_edge_with_flow(a, g.sink, Capacity(1), Cost(4), Flow(1));
        g.add_edge_with_flow(b, g.sink, Capacity(6), Cost(1), Flow(5));

        let g = g.residual_graph().0;
        assert_eq!(
            r#"digraph g {
"()" -> "0" [label="0/2;-1"];
"()" -> "0" [label="0/4;-1"];
"()" -> "()" [label="0/2;1"];
"()" -> "()" [label="0/1;-1"];
"1" -> "()" [label="0/1;-4"];
"()" -> "1" [label="0/1;1"];
"1" -> "()" [label="0/5;-1"];

}"#,
            g.graphviz()
        );
        assert_eq!(g.edges[0].capacity.0, 2);
    }

    #[test]
    fn paths() {
        let mut g = Graph::new();
        let a = g.add_node(1);
        let b = g.add_node(2);
        let c = g.add_node(3);

        g.add_edge_with_flow(g.source, a, Capacity(2), Cost(1), Flow(2));
        g.add_edge_with_flow(g.source, b, Capacity(4), Cost(1), Flow(4));
        g.add_edge_with_flow(a, g.sink, Capacity(1), Cost(4), Flow(1));
        g.add_edge_with_flow(b, g.sink, Capacity(6), Cost(1), Flow(5));
        g.add_edge_with_flow(g.source, c, Capacity(4), Cost(1), Flow(0));

        let paths = g.paths();
        assert_eq!(paths[0].edges[0].flow, 4);
        assert_eq!(paths[0].edges[1].flow, 5);
        assert_eq!(paths[1].edges[0].flow, 2);
        assert_eq!(paths[1].edges[1].flow, 1);
    }
}
