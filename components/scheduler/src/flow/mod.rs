
mod resource_profile;

use self::resource_profile::ResourceProfile;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use mcmf::{GraphBuilder, Vertex, Cost, Capacity, Path};


#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub struct Task {
    request: ResourceProfile,
}

impl Task {
    fn with_resource_profile(profile: ResourceProfile) -> Self {
        Self {
            request: profile,
        }
    }

    fn get_request(&self) -> &ResourceProfile {
        &self.request
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub struct Server {
    current: ResourceProfile,
}

impl Server {
    fn with_resource_profile(profile: ResourceProfile) -> Self {
        Self {
            current: profile,
        }
    }

    fn get_request(&self) -> &ResourceProfile {
        &self.current
    }
}


#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub struct VirtualResource {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum Node {
    VirtualResource(VirtualResource),
    Server(Server),
    Task(Task),
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

    fn count_nodes(&self) -> usize {
        self.tasks.len() + self.servers.len()
    }

    pub fn run(&self) -> Vec<Path<Node>> {
        let mut graph = GraphBuilder::<Node>::new();
        let cluster = Node::VirtualResource(VirtualResource {});
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
            graph.add_edge(Node::Server(server.clone()), Vertex::Sink, Capacity(task_count as i32), Cost(0));
        }
        graph.mcmf().1
    }
}