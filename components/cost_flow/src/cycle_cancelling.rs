use super::*;
use crate::FordFulkerson;
use crate::BFS;
use std::cmp::min;

pub trait CycleCancelling {
    fn cycle_cancelling(&mut self);
    fn bellman_ford(&self) -> Option<Vec<EdgeData>>;
}

#[derive(Clone)]
enum ToParent {
    None,
    OverEdge(EdgeIndex),
}

impl<T: Clone> CycleCancelling for Graph<T> {
    fn cycle_cancelling(&mut self) {
        let (flow, mut graph) = self.ford_fulkerson();
        while let Some(cycle) = graph.bellman_ford() {
            let min_edge = cycle.iter().min_by_key(|x| x.capacity);
            for edge in cycle {
                if edge == min_edge {
                    edge.capacity = Capacity(0);
                } else {
                    e
                }
            }
        }
    }

    fn bellman_ford(&self) -> Option<Vec<EdgeData>> {
        let mut distance = vec![Cost::MAX; self.nodes.len()];
        let mut parent = vec![ToParent::None; self.nodes.len()];

        distance[self.source.0] = Cost(0);

        for _ in 0..self.nodes.len() - 1 {
            for (i, edge) in self.edges.iter().enumerate() {
                let u = edge.source.0;
                let v = edge.target.0;
                if distance[u] != Cost::MAX && edge.capacity > Capacity(0) && distance[u] + edge.cost < distance[v] {
                    distance[v] = distance[u] + edge.cost;
                    parent[v] = ToParent::OverEdge(EdgeIndex(i));
                }
            }
        }

        for (i, edge) in self.edges.clone().iter().enumerate() {
            let u = edge.source.0;
            let v = edge.target.0;
            if distance[u] != Cost::MAX && edge.capacity > Capacity(0) && distance[u] + edge.cost < distance[v] {
                let mut edge_i = edge;
                let mut cycle_edges = vec![];
                loop {
                    cycle_edges.push(edge_i.clone());
                    edge_i = match parent[edge_i.source.0] {
                        ToParent::OverEdge(e) => &self.edges[e.0],
                        _ => unreachable!(),
                    };
                    if edge == edge_i {
                        break;
                    }
                }
                return Some(cycle_edges);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cycle() {
        let mut graph = Graph::new();

        let a = graph.add_node(1);
        let b = graph.add_node(2);

        graph.add_edge(graph.source, a, Capacity(1), Cost(-4));
        graph.add_edge(graph.source, b, Capacity(5), Cost(-1));
        graph.add_edge(b, graph.source, Capacity(1), Cost(1));
        graph.add_edge(a, b, Capacity(2), Cost(1));
        graph.add_edge(b, a, Capacity(1), Cost(-1));
        graph.add_edge(a, graph.sink, Capacity(2), Cost(-1));
        graph.add_edge(b, graph.sink, Capacity(4), Cost(-1));

        assert_eq!(graph.bellman_ford().unwrap().len(), 3);
    }
}
