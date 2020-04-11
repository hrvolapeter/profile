use super::*;
use std::collections::VecDeque;

pub trait BFS {
    /// Returns edge indices if there is a path from source 's' to sink 't' in
    /// graph.
    fn bfs(&self) -> Option<Path>;

    /// Convenience wrapper around `bfs()` returning edges instead of indeaces
    fn bfs_path(&self) -> Option<Vec<EdgeData>>;
}

#[derive(Clone)]
enum ToParent {
    None,
    OverEdge(EdgeIndex),
}

impl<T> BFS for Graph<T> {
    fn bfs(&self) -> Option<Path> {
        let mut visited = vec![false; self.nodes.len()];
        let mut parent = vec![ToParent::None; self.nodes.len()];

        let mut q = VecDeque::new();
        q.push_back(self.source);
        visited[self.source.0] = true;

        while let Some(first) = q.pop_front() {
            for edge in self.edges(first) {
                if !visited[edge.target.0] && edge.flow.0 != edge.capacity.0 {
                    q.push_back(edge.target);
                    visited[edge.target.0] = true;
                    parent[edge.target.0] = ToParent::OverEdge(edge.index);
                }
            }
        }

        let mut i = self.sink.0;
        let mut path = vec![];
        while let ToParent::OverEdge(edge) = &parent[i] {
            path.push(edge.clone());
            i = self.edges[edge.0].source.0;
        }
        path.reverse();
        if i == self.source.0 { Some(Path { path }) } else { None }
    }

    fn bfs_path(&self) -> Option<Vec<EdgeData>> {
        self.bfs().map(|x| x.path.iter().map(|x| self.edges[x.0].clone()).collect())
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
        graph.add_edge(graph.source, a, Capacity(1), Cost(0));
        graph.add_edge(a, b, Capacity(1), Cost(1));
        graph.add_edge(b, c, Capacity(2), Cost(2));
        graph.add_edge(a, d, Capacity(3), Cost(3));
        graph.add_edge(b, d, Capacity(3), Cost(3));
        graph.add_edge(d, c, Capacity(3), Cost(3));
        graph.add_edge(c, graph.sink, Capacity(1), Cost(0));

        let res = graph.bfs_path();
        let nodes: Vec<_> = res.unwrap().iter().map(|x| x.source).collect();
        assert_eq!(nodes, vec![graph.source, a, d, c]);
    }
}
