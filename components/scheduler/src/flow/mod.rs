mod resource_profile;

use self::resource_profile::ResourceProfile;
use mcmf::{Capacity, Cost, Flow, GraphBuilder, Vertex};
use serde::Serialize;
use tokio::sync::watch;

pub trait Displayable {
    fn name(&self) -> String;
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub struct Task {
    request: ResourceProfile,
    name: String,
}

impl Task {
    pub fn new(name: String, request: ResourceProfile) -> Self {
        Self { name, request }
    }

    fn get_request(&self) -> &ResourceProfile {
        &self.request
    }
}

impl Displayable for Task {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub struct Server {
    name: String,
    current: ResourceProfile,
}

impl Server {
    pub fn new(name: String, current: ResourceProfile) -> Self {
        Self { name, current }
    }

    fn get_request(&self) -> &ResourceProfile {
        &self.current
    }
}

impl Displayable for Server {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub struct VirtualResource {
    name: String,
}

impl VirtualResource {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Displayable for VirtualResource {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub enum Node {
    VirtualResource(VirtualResource),
    Server(Server),
    Task(Task),
}

impl Displayable for Node {
    fn name(&self) -> String {
        match self {
            Node::VirtualResource(t) => t.name(),
            Node::Server(t) => t.name(),
            Node::Task(t) => t.name(),
        }
    }
}

type Flows = Vec<Flow<Node>>;

pub struct Scheduler {
    tasks: Vec<Task>,
    servers: Vec<Server>,
    notif_channel: (watch::Sender<Flows>, watch::Receiver<Flows>),
    flows: Flows,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            notif_channel: watch::channel(vec![]),
            tasks: Default::default(),
            servers: Default::default(),
            flows: Default::default(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn add_server(&mut self, server: Server) {
        self.servers.push(server);
    }

    pub fn get_servers(&self) -> &Vec<Server> {
        &self.servers
    }

    fn get_schedule(&mut self) {
        let graph = self.build_flow_graph();
        let (_, _, flows) = graph.mcmf();
        self.flows = flows;
    }

    fn build_flow_graph(&self) -> GraphBuilder<Node> {
        let mut graph = GraphBuilder::new();
        let cluster = Node::VirtualResource(VirtualResource::new("Cluster".to_string()));
        let task_count = self.tasks.len();

        for task in &self.tasks {
            graph.add_edge(Vertex::Source, Node::Task(task.clone()), Capacity(1), Cost(0));
            graph.add_edge(
                Node::Task(task.clone()),
                cluster.clone(),
                Capacity(1),
                Cost(task.get_request().inner_product() as i32),
            );
        }

        for server in &self.servers {
            graph.add_edge(
                cluster.clone(),
                Node::Server(server.clone()),
                Capacity(task_count as i32),
                Cost(server.get_request().inner_product() as i32),
            );
            graph.add_edge(
                Node::Server(server.clone()),
                Vertex::Sink,
                Capacity(task_count as i32),
                Cost(0),
            );
        }
        graph
    }

    pub fn subscribe(&self) -> watch::Receiver<Flows> {
        self.notif_channel.1.clone()
    }
}
