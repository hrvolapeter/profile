#![deny(warnings)]
#![deny(clippy::pedantic)]
#![allow(clippy::new_without_default)]
#![allow(clippy::wildcard_imports)]

mod bfs;
mod cycle_cancelling;
mod ford_fulkerson;

pub use bfs::BFS;
pub use cycle_cancelling::CycleCancelling;
pub use ford_fulkerson::FordFulkerson;

#[derive(Clone)]
pub struct Graph<T> {
    nodes: Vec<NodeData<T>>,
    edges: Vec<EdgeData>,
    pub source: NodeIndex,
    pub sink: NodeIndex,
}

// NODE
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct NodeIndex(usize);

#[derive(Clone)]
struct NodeData<T> {
    first_outgoing_edge: Option<EdgeIndex>,
    pub inner: Node<T>,
}

#[derive(Clone)]
pub enum Node<T> {
    Sink,
    Source,
    Node(T),
}

// EDGE

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct EdgeIndex(usize);

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct Cost(i64);

impl Cost {
    const MAX: Cost = Self(i64::MAX);
}

impl std::ops::Add for Cost {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct Capacity(i64);

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

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct Flow(i64);

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

#[derive(Clone, PartialEq, Debug)]
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

pub struct Path {
    pub path: Vec<EdgeIndex>,
}

impl<T> Graph<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: vec![
                NodeData { first_outgoing_edge: None, inner: Node::Source },
                NodeData { first_outgoing_edge: None, inner: Node::Sink },
            ],
            edges: vec![],
            source: NodeIndex(0),
            sink: NodeIndex(1),
        }
    }

    #[must_use]
    pub fn add_node(&mut self, inner: T) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(NodeData { first_outgoing_edge: None, inner: Node::Node(inner) });
        NodeIndex(index)
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

    fn residual_graph(&self) -> (Graph<()>, Vec<EdgeIndex>) {
        let mut res = Graph::new();
        for _ in &self.nodes {
            let _ = res.add_node(());
        }
        let mut mapping = vec![];
        for edge in &self.edges {
            if edge.capacity.0 != edge.flow.0 {
                mapping.push(edge.index);
                res.add_edge(edge.source, edge.target, edge.residual_capacity(), edge.cost);
            }
            mapping.push(edge.index);
            res.add_edge(edge.target, edge.source, Capacity(edge.flow.0), Cost(-edge.cost.0));
        }
        (res, mapping)
    }

    #[must_use]
    pub fn successors(&self, source: NodeIndex) -> Successors<T> {
        let first_outgoing_edge = self.nodes[source.0].first_outgoing_edge;
        Successors { graph: self, current_edge_index: first_outgoing_edge }
    }

    #[must_use]
    pub fn edges(&self, source: NodeIndex) -> Edges<T> {
        let first_outgoing_edge = self.nodes[source.0].first_outgoing_edge;
        Edges { graph: self, current_edge_index: first_outgoing_edge }
    }
}

pub struct Successors<'graph, T> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, T> Iterator for Successors<'graph, T> {
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

pub struct Edges<'graph, T> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, T> Iterator for Edges<'graph, T> {
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

    #[test]
    fn empty_graph() {
        Graph::<i8>::new();
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
        let a = g.add_node(1);
        let b = g.add_node(2);

        g.add_edge_with_flow(g.source, a, Capacity(2), Cost(1), Flow(2));
        g.add_edge_with_flow(g.source, b, Capacity(4), Cost(1), Flow(4));
        g.add_edge_with_flow(a, b, Capacity(3), Cost(1), Flow(1));
        g.add_edge_with_flow(a, g.sink, Capacity(1), Cost(4), Flow(1));
        g.add_edge_with_flow(b, g.sink, Capacity(6), Cost(1), Flow(5));

        let g = g.residual_graph().0;
        assert_eq!(g.edges[0].capacity.0, 2);
        assert_eq!(g.edges[0].cost.0, -1);
        assert_eq!(g.edges[1].capacity.0, 4);
        assert_eq!(g.edges[1].cost.0, -1);
        assert_eq!(g.edges[2].capacity.0, 2);
        assert_eq!(g.edges[2].cost.0, 1);
        assert_eq!(g.edges[3].capacity.0, 1);
        assert_eq!(g.edges[3].cost.0, -1);
        assert_eq!(g.edges[4].capacity.0, 1);
        assert_eq!(g.edges[4].cost.0, -4);
        assert_eq!(g.edges[5].capacity.0, 1);
        assert_eq!(g.edges[5].cost.0, 1);
        assert_eq!(g.edges[6].capacity.0, 5);
        assert_eq!(g.edges[6].cost.0, -1);
    }
}
