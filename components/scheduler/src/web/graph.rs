use serde::{Deserialize, Serialize};
use pharos::{Events, Observable, ObserveConfig, Pharos};
use crate::flow::FlowGraph;


#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Node {
    id: u32,
    label: String,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Edge {
    from: u32,
    to: u32,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
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

impl From<FlowGraph> for Graph {
    fn from(from: FlowGraph) -> Self {
        let flow = from.dinic(1, 2).1;
        for edge in from.graph.adj_list() {

        }
    }
}