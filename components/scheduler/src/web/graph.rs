use pharos::{Events, Observable, ObserveConfig, Pharos};
use serde::{Deserialize, Serialize};
use mcmf::Flow;
use mcmf::Vertex;
use crate::flow::Node as FlowNode;
use crate::flow::Displayable;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

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
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn from_flow(paths: Vec<Flow<FlowNode>>)  -> Self {
        let note_id: AtomicUsize = AtomicUsize::new(0);
        let mut nodes = HashMap::new();
        let mut edges = vec![];

        for path in &paths {
            for vertex in &path.vertices() {
                if nodes.contains_key(vertex) {
                    continue;
                }
                let label = match vertex {
                    Vertex::Node(t) => t.name(),
                    Vertex::Source => "Source".to_string(),
                    Vertex::Sink => "Sink".to_string(),
                };
                nodes.insert(vertex.clone(), Node{
                    id: note_id.fetch_add(1, Ordering::Relaxed) as u32,
                    label: label,
                });
            }
            for edge in &path.edges() {
                dbg!(&edge.a);
                dbg!(&edge.b);
                println!("#####");

                let a = nodes.get(&edge.a).unwrap();
                let b = nodes.get(&edge.b).unwrap();
                edges.push(Edge {
                    from: a.id,
                    to: b.id,
                    label: format!("fl:{} cst:{}", edge.amount, edge.cost),
                });
            }
        }
        Graph {
            edges,
            nodes: nodes.into_iter().map(|(_,v)| v).collect(),
        }
    }
}

pub struct GraphEvent {
    pharos: Pharos<Graph>,
}

impl Observable<Graph> for GraphEvent {
    type Error = pharos::Error;

    fn observe(&mut self, options: ObserveConfig<Graph>) -> Result<Events<Graph>, Self::Error> {
        self.pharos.observe(options)
    }
}

