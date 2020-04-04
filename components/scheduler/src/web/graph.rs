use crate::flow::Displayable;
use crate::flow::Node as FlowNode;
use crate::import::*;
use mcmf::Flow;
use mcmf::Vertex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Deserialize, Serialize, Default, Clone, Hash, PartialEq, Eq)]
pub struct Node {
    id: u32,
    label: String,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Edge {
    from: u32,
    to: u32,
    label: String,
    value: u32,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn from_flow(flows: Vec<Flow<FlowNode>>) -> Self {
        let note_id: AtomicUsize = AtomicUsize::new(0);
        let mut nodes = HashMap::new();
        let mut edges = HashMap::<(_, _), Flow<FlowNode>>::new();

        let mut insert_node = |node: &Vertex<FlowNode>| {
            if nodes.contains_key(node) {
                return;
            }
            let label = match node {
                Vertex::Node(t) => t.name(),
                Vertex::Source => "Source".to_string(),
                Vertex::Sink => "Sink".to_string(),
            };
            nodes.insert(
                node.clone(),
                Node { id: note_id.fetch_add(1, Ordering::Relaxed) as u32, label },
            );
        };

        for flow in flows {
            insert_node(&flow.a);
            insert_node(&flow.b);
            let key = (flow.a.clone(), flow.b.clone());
            if edges.contains_key(&key) {
                let mut edge = edges.get_mut(&key).unwrap();
                edge.amount += flow.amount;
                edge.cost += flow.cost;
            } else {
                edges.insert(key.clone(), flow);
            }
        }

        Graph {
            edges: edges
                .into_iter()
                .map(|(_, flow)| {
                    let a = nodes.get(&flow.a).unwrap();
                    let b = nodes.get(&flow.b).unwrap();
                    Edge {
                        from: a.id,
                        to: b.id,
                        label: format!("fl:{} cst:{}", flow.amount, flow.cost),
                        value: flow.amount,
                    }
                })
                .collect(),
            nodes: nodes.into_iter().map(|(_, v)| v).collect(),
        }
    }
}
