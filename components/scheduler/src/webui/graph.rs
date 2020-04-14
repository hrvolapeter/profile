use crate::import::*;
use crate::scheduler;
use crate::scheduler::Displayable;
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
    value: u64,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn from_flow(flows: Vec<cost_flow::Edge<scheduler::Node>>) -> Self {
        let note_id: AtomicUsize = AtomicUsize::new(0);
        let mut nodes = HashMap::new();
        let mut edges = HashMap::<(_, _), cost_flow::Edge<scheduler::Node>>::new();

        let mut insert_node = |node: &cost_flow::Node<scheduler::Node>| {
            if nodes.contains_key(node) {
                return;
            }
            let label = match node {
                cost_flow::Node::Node(t) => t.name(),
                cost_flow::Node::Source => "Source".to_string(),
                cost_flow::Node::Sink => "Sink".to_string(),
            };
            nodes.insert(
                node.clone(),
                Node { id: note_id.fetch_add(1, Ordering::Relaxed) as u32, label },
            );
        };

        for flow in flows {
            insert_node(&flow.source);
            insert_node(&flow.target);
            let key = (flow.source.clone(), flow.target.clone());
            edges.insert(key, flow);
        }

        Graph {
            edges: edges
                .into_iter()
                .map(|(_, flow)| {
                    let a = nodes.get(&flow.source).unwrap();
                    let b = nodes.get(&flow.target).unwrap();
                    let cost = if flow.cost as i64 == i64::MAX {
                        "inf".to_string()
                    } else {
                        format!("{}", flow.cost)
                    };
                    Edge {
                        from: a.id,
                        to: b.id,
                        label: format!("fl:{} cst:{}", flow.flow, cost),
                        value: flow.flow,
                    }
                })
                .collect(),
            nodes: nodes.into_iter().map(|(_, v)| v).collect(),
        }
    }
}
