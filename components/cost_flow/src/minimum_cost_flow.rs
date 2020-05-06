use super::*;
use std::collections::HashSet;

pub trait MinimumCostFlow {
    fn minimum_cost_flow(&mut self);
    fn bellman_ford(&self) -> Option<Vec<EdgeData>>;
}

#[derive(Clone)]
enum ToParent {
    None,
    OverEdge(EdgeIndex),
}

impl<T: Clone + Debug + Graphable> MinimumCostFlow for Graph<T> {
    fn minimum_cost_flow(&mut self) {
        self.ford_fulkerson();
        loop {
            let (residual, res_index_to_g_index) = self.residual_graph();
            if let Some(cycle) = residual.bellman_ford() {
                let min_edge = cycle.iter().min_by_key(|x| x.capacity.0).unwrap();
                for edge in &cycle {
                    let transpose = res_index_to_g_index[edge.index.0];
                    match transpose {
                        Ok(i) => self.edges[i.0].flow += Flow(min_edge.capacity.0),
                        Err(i) => self.edges[i.0].flow -= Flow(min_edge.capacity.0),
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
                if distance[u] != Cost::MAX
                    && distance[u] + edge.cost < distance[v]
                    && edge.capacity != Capacity(0)
                {
                    distance[v] = distance[u] + edge.cost;
                    parent[v] = ToParent::OverEdge(edge.index);
                }
            }
        }

        for edge in &self.edges.clone() {
            let u = edge.source.0;
            let v = edge.target.0;
            let diff = (distance[u] + edge.cost).0.checked_sub(distance[v].0).unwrap_or(i64::MIN);

            if distance[u] != Cost::MAX && edge.capacity != Capacity(0) && diff < 0 {
                // 1. Find beginning of the cycle
                let mut edge_i = edge;
                let mut cycle_edges = HashSet::new();
                loop {
                    edge_i = match parent[edge_i.source.0] {
                        ToParent::OverEdge(e) => &self.edges[e.0],
                        _ => unreachable!(),
                    };
                    if !cycle_edges.insert(edge_i.clone()) {
                        break;
                    }
                }
                // 2. Augment the cycle
                let mut cycle_edges = HashSet::new();
                loop {
                    edge_i = match parent[edge_i.source.0] {
                        ToParent::OverEdge(e) => &self.edges[e.0],
                        _ => unreachable!(),
                    };
                    if !cycle_edges.insert(edge_i.clone()) {
                        break;
                    }
                }
                return Some(cycle_edges.into_iter().collect());
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn cycle() {
        let mut graph = Graph::new();

        let a = graph.add_node(2);
        let b = graph.add_node(3);

        graph.add_edge(graph.sink, a, Capacity(1), Cost(-4));
        graph.add_edge(graph.sink, b, Capacity(5), Cost(-1));
        graph.add_edge(b, graph.sink, Capacity(1), Cost(1));
        graph.add_edge(a, b, Capacity(2), Cost(1));
        graph.add_edge(b, a, Capacity(1), Cost(-1));
        graph.add_edge(a, graph.source, Capacity(2), Cost(-1));
        graph.add_edge(b, graph.source, Capacity(4), Cost(-1));

        assert_eq!(graph.bellman_ford().unwrap().len(), 3);
    }

    #[test]
    fn complex() {
        let mut graph = Graph::new();

        let a = graph.add_node(2);
        let b = graph.add_node(3);

        graph.add_edge(graph.source, a, Capacity(2), Cost(1));
        graph.add_edge(graph.source, b, Capacity(4), Cost(1));
        graph.add_edge(a, b, Capacity(3), Cost(1));
        graph.add_edge(a, graph.sink, Capacity(1), Cost(4));
        graph.add_edge(b, graph.sink, Capacity(6), Cost(1));

        graph.minimum_cost_flow();
        assert_eq!(
            r#"digraph g {
"0" -> "2" [label="2/2;1"];
"0" -> "3" [label="4/4;1"];
"2" -> "3" [label="2/3;1"];
"2" -> "1" [label="0/1;4"];
"3" -> "1" [label="6/6;1"];

}"#,
            graph.graphviz()
        );
    }

    #[test]
    fn scheduling_simple() {
        let mut graph = Graph::new();

        let task1 = graph.add_node(2);
        let task2 = graph.add_node(3);
        let task3 = graph.add_node(4);
        let cluster = graph.add_node(5);
        let unscheduled1 = graph.add_node(6);
        let unscheduled2 = graph.add_node(7);
        let unscheduled3 = graph.add_node(8);
        let server = graph.add_node(9);

        graph.add_edge(graph.source, task1, Capacity(1), Cost(0));
        graph.add_edge(graph.source, task2, Capacity(1), Cost(0));
        graph.add_edge(graph.source, task3, Capacity(1), Cost(0));
        graph.add_edge(task1, cluster, Capacity(1), Cost(0));
        graph.add_edge(task2, cluster, Capacity(1), Cost(0));
        graph.add_edge(task3, cluster, Capacity(1), Cost(0));
        graph.add_edge(task1, unscheduled1, Capacity(1), Cost(0));
        graph.add_edge(task2, unscheduled2, Capacity(1), Cost(0));
        graph.add_edge(task3, unscheduled3, Capacity(1), Cost(0));
        graph.add_edge(unscheduled1, graph.sink, Capacity(1), Cost(800));
        graph.add_edge(unscheduled2, graph.sink, Capacity(1), Cost(800));
        graph.add_edge(unscheduled3, graph.sink, Capacity(1), Cost(800));
        graph.add_edge(cluster, server, Capacity(3), Cost(400));
        graph.add_edge(server, graph.sink, Capacity(3), Cost(1));

        println!("{}", graph.graphviz());
        graph.minimum_cost_flow();

        assert_eq!(
            r#"digraph g {
"0" -> "2" [label="1/1;0"];
"0" -> "3" [label="1/1;0"];
"0" -> "4" [label="1/1;0"];
"2" -> "5" [label="1/1;0"];
"3" -> "5" [label="1/1;0"];
"4" -> "5" [label="1/1;0"];
"2" -> "6" [label="0/1;0"];
"3" -> "7" [label="0/1;0"];
"4" -> "8" [label="0/1;0"];
"6" -> "1" [label="0/1;800"];
"7" -> "1" [label="0/1;800"];
"8" -> "1" [label="0/1;800"];
"5" -> "9" [label="3/3;400"];
"9" -> "1" [label="3/3;1"];

}"#,
            graph.graphviz()
        );
    }

    #[test]
    fn regression_01() {
        let mut graph = Graph::new();

        let cluster = graph.add_node("Cluster");
        let dionysos = graph.add_node("dionysos");
        let dasya1 = graph.add_node("dasya1");
        let cpu = graph.add_node("cpu");
        let unscheduled_cpu = graph.add_node("Unscheduled cpu");
        let cpub = graph.add_node("cpub");
        let unscheduled_cpub = graph.add_node("Unscheduled cpub");

        graph.add_edge(cluster, dionysos, Capacity(2), Cost(93));
        graph.add_edge(dionysos, graph.sink, Capacity(2), Cost(0));
        graph.add_edge(cluster, dasya1, Capacity(2), Cost(166));
        graph.add_edge(dasya1, graph.sink, Capacity(2), Cost(0));
        graph.add_edge(graph.source, cpu, Capacity(1), Cost(0));
        graph.add_edge(cpu, cluster, Capacity(1), Cost(0));
        graph.add_edge(cpu, unscheduled_cpu, Capacity(1), Cost(0));
        graph.add_edge(unscheduled_cpu, graph.sink, Capacity(1), Cost(1000));
        graph.add_edge(graph.source, cpub, Capacity(1), Cost(0));
        graph.add_edge(cpub, cluster, Capacity(1), Cost(0));
        graph.add_edge(cpub, dionysos, Capacity(1), Cost(0));
        graph.add_edge(cpub, unscheduled_cpub, Capacity(1), Cost(0));
        graph.add_edge(unscheduled_cpub, graph.sink, Capacity(1), Cost(1000));

        graph.minimum_cost_flow();
        assert_eq!(
            "digraph g {
\"Cluster\" -> \"dionysos\" [label=\"1/2;93\"];
\"dionysos\" -> \"1\" [label=\"2/2;0\"];
\"Cluster\" -> \"dasya1\" [label=\"0/2;166\"];
\"dasya1\" -> \"1\" [label=\"0/2;0\"];
\"0\" -> \"cpu\" [label=\"1/1;0\"];
\"cpu\" -> \"Cluster\" [label=\"1/1;0\"];
\"cpu\" -> \"Unscheduled cpu\" [label=\"0/1;0\"];
\"Unscheduled cpu\" -> \"1\" [label=\"0/1;1000\"];
\"0\" -> \"cpub\" [label=\"1/1;0\"];
\"cpub\" -> \"Cluster\" [label=\"0/1;0\"];
\"cpub\" -> \"dionysos\" [label=\"1/1;0\"];
\"cpub\" -> \"Unscheduled cpub\" [label=\"0/1;0\"];
\"Unscheduled cpub\" -> \"1\" [label=\"0/1;1000\"];

}",
            graph.graphviz()
        );
    }
}
