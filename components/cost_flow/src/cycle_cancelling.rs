use super::*;

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
        self.ford_fulkerson();
        loop {
            let (residual, res_index_to_g_index) = self.residual_graph();
            if let Some(cycle) = residual.bellman_ford() {
                let min_edge = cycle.iter().min_by_key(|x| x.capacity).unwrap();
                for edge in &cycle {
                    if edge == min_edge {
                        self.edges[res_index_to_g_index[edge.index.0].0].flow -=
                            Flow(min_edge.capacity.0);
                    } else {
                        self.edges[res_index_to_g_index[edge.index.0].0].flow +=
                            Flow(min_edge.capacity.0);
                    }
                }
            } else {
                break;
            }
        }
    }

    fn bellman_ford(&self) -> Option<Vec<EdgeData>> {
        let mut distance = vec![Cost::MAX; self.nodes.len()];
        let mut parent = vec![ToParent::None; self.nodes.len()];

        distance[self.sink.0] = Cost(0);

        for _ in 0..self.nodes.len() - 1 {
            for edge in &self.edges {
                let u = edge.source.0;
                let v = edge.target.0;
                if distance[u] != Cost::MAX && distance[u] + edge.cost < distance[v] {
                    distance[v] = distance[u] + edge.cost;
                    parent[v] = ToParent::OverEdge(edge.index);
                }
            }
        }

        for edge in &self.edges.clone() {
            let u = edge.source.0;
            let v = edge.target.0;
            let diff = (distance[u] + edge.cost).0 - distance[v].0;

            if distance[u] != Cost::MAX && diff < 0 {
                let mut edge_i = edge;
                let mut cycle_edges = vec![];
                let mut diff_traversed_back = 0;
                loop {
                    edge_i = match parent[edge_i.source.0] {
                        ToParent::OverEdge(e) => &self.edges[e.0],
                        _ => unreachable!(),
                    };
                    cycle_edges.push(edge_i.clone());
                    diff_traversed_back += edge_i.cost.0;
                    if diff == diff_traversed_back {
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

        graph.add_edge(graph.sink, a, Capacity(1), Cost(-4));
        graph.add_edge(graph.sink, b, Capacity(5), Cost(-1));
        graph.add_edge(b, graph.sink, Capacity(1), Cost(1));
        graph.add_edge(a, b, Capacity(2), Cost(1));
        graph.add_edge(b, a, Capacity(1), Cost(-1));
        graph.add_edge(a, graph.source, Capacity(2), Cost(-1));
        graph.add_edge(b, graph.source, Capacity(4), Cost(-1));

        assert_eq!(graph.clone().bellman_ford().unwrap().len(), 3);
    }

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

        graph.cycle_cancelling();
        assert_eq!(graph.edges[4].flow.0, 6);
        assert_eq!(graph.edges[4].capacity.0, 6);
        assert_eq!(graph.edges[3].flow.0, 0);
        assert_eq!(graph.edges[3].capacity.0, 1);
        assert_eq!(graph.edges[2].flow.0, 2);
        assert_eq!(graph.edges[2].capacity.0, 3);
        assert_eq!(graph.edges[1].flow.0, 4);
        assert_eq!(graph.edges[1].capacity.0, 4);
        assert_eq!(graph.edges[0].flow.0, 2);
        assert_eq!(graph.edges[0].capacity.0, 2);
    }
}
