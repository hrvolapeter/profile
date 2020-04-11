use super::*;
use crate::BFS;
use std::cmp::min;

pub trait FordFulkerson {
    /// Returns path if there is a path from source 's' to sink 't' in
    /// graph
    fn ford_fulkerson(&self) -> (Capacity, Self);
}

impl<T: Clone> FordFulkerson for Graph<T> {
    fn ford_fulkerson(&self) -> (Capacity, Self) {
        let mut residue = (*self).clone();
        let mut max_flow = Capacity(0);

        while let Some(path) = residue.bfs() {
            let mut path_capacity = Capacity::MAX;
            for edge in &path.path {
                path_capacity = min(path_capacity, residue.edges[edge.0].capacity);
            }

            for edge in path.path {
                let edge = &mut residue.edges[edge.0];
                edge.capacity -= path_capacity;
                let edge = edge.clone();
                residue.add_edge(edge.target, edge.source, path_capacity, Cost(-edge.cost.0));
            }

            max_flow += path_capacity;
        }

        (max_flow, residue)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_graph() {
        let mut graph = Graph::new();

        let a = graph.add_node(1);
        let b = graph.add_node(2);
        let c = graph.add_node(3);
        let d = graph.add_node(4);
        graph.add_edge(graph.source, a, Capacity(2), Cost(0));
        graph.add_edge(a, b, Capacity(1), Cost(1));
        graph.add_edge(b, c, Capacity(2), Cost(2));
        graph.add_edge(b, d, Capacity(3), Cost(3));
        graph.add_edge(d, c, Capacity(3), Cost(3));
        graph.add_edge(c, graph.sink, Capacity(2), Cost(0));

        assert_eq!(graph.ford_fulkerson().0, Capacity(1));

        graph.add_edge(a, d, Capacity(2), Cost(1));

        assert_eq!(graph.ford_fulkerson().0, Capacity(2));
    }
}
