
mod resource_profile;

use self::resource_profile::ResourceProfile;


use mcmf::{GraphBuilder, Vertex, Cost, Capacity, Path};

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
        Self {
            name,
            request,
        }
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

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub struct Server {
    name: String,
    current: ResourceProfile,
}

impl Server {
    pub fn new(name: String, current: ResourceProfile) -> Self {
        Self {
            name,
            current,
        }
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
        Self {
            name,
        }
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


#[derive(Default)]
pub struct Graph {
    tasks: Vec<Task>,
    servers: Vec<Server>,
}

impl Graph {
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn add_server(&mut self, server: Server) {
        self.servers.push(server);
    }

    pub fn run(&self) -> Vec<Path<Node>> {
        let mut graph = GraphBuilder::<Node>::new();
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
            dbg!(server);
            graph.add_edge(
                cluster.clone(),
                Node::Server(server.clone()),
                Capacity(task_count as i32),
                Cost(server.get_request().inner_product() as i32),
            );
            graph.add_edge(Node::Server(server.clone()), Vertex::Sink, Capacity(task_count as i32), Cost(0));
        }
        graph.mcmf().2
    }
}