use super::*;
use std::cmp::min;

pub trait FordFulkerson {
    /// Returns path if there is a path from source 's' to sink 't' in
    /// graph. Is idempotent
    fn ford_fulkerson(&mut self);
}

impl<T: Clone + Debug> FordFulkerson for Graph<T> {
    fn ford_fulkerson(&mut self) {
        while let Some(path) = self.bfs() {
            let mut residual_path_capacity = Capacity::MAX;
            for edge in &path.0 {
                residual_path_capacity =
                    min(residual_path_capacity, self.edges[edge.0].residual_capacity());
            }
            for edge in path.0 {
                let edge = &mut self.edges[edge.0];
                edge.flow += Flow(residual_path_capacity.0);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn complex() {
        let mut graph = Graph::new();

        let a = graph.add_node(1);
        let b = graph.add_node(2);

        graph.add_edge(graph.source, a, Capacity(2), Cost(1));
        graph.add_edge(graph.source, b, Capacity(4), Cost(1));
        graph.add_edge(a, b, Capacity(3), Cost(1));
        graph.add_edge(a, graph.sink, Capacity(1), Cost(4));
        graph.add_edge(b, graph.sink, Capacity(6), Cost(1));

        graph.ford_fulkerson();
        assert_eq!(graph.edges[0].flow.0, 2);
        assert_eq!(graph.edges[1].flow.0, 4);
        assert_eq!(graph.edges[2].flow.0, 1);
        assert_eq!(graph.edges[3].flow.0, 1);
        assert_eq!(graph.edges[4].flow.0, 5);

        // indempotence
        graph.ford_fulkerson();
        assert_eq!(graph.edges[0].flow.0, 2);
        assert_eq!(graph.edges[1].flow.0, 4);
        assert_eq!(graph.edges[2].flow.0, 1);
        assert_eq!(graph.edges[3].flow.0, 1);
        assert_eq!(graph.edges[4].flow.0, 5);
    }
}
