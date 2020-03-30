pub mod solver;

use self::solver::resource_profile::ResourceProfile;
pub use self::solver::flow::FlowGraph;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;


static NEXT_NODE_ID: AtomicUsize = AtomicUsize::new(0);

trait Node {
    fn get_id(&self) -> usize;
}


pub struct Task {
    id: usize,
   request: ResourceProfile,
}

impl Task {
    fn new() -> Self {
        Self::with_resource_profile(Default::default())
    }

    fn with_resource_profile(profile: ResourceProfile) -> Self {
        Self {
            id: NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed),
            request: profile,
        }
    }

    fn get_request(&self) -> &ResourceProfile {
        &self.request
    }
}

impl Node for Task {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub struct Server {
    id: usize,
    current: ResourceProfile,
}

impl Server {
    fn new() -> Self {
        Self::with_resource_profile(Default::default())
    }

    fn with_resource_profile(profile: ResourceProfile) -> Self {
        Self {
            id: NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed),
            current: profile,
        }
    }

    fn get_request(&self) -> &ResourceProfile {
        &self.current
    }
}

impl Node for Server {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub struct VirtualResource {
    id: usize,
}

impl VirtualResource {
    fn new() -> Self {
        Self {
            id: NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Node for VirtualResource {
    fn get_id(&self) -> usize {
        self.id
    }
} 

#[derive(Default)]
pub struct Graph {
    tasks: Vec<Task>,
    servers: Vec<Server>,
}

impl Graph {
    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    fn add_server(&mut self, server: Server) {
        self.servers.push(server);
    }

    fn count_nodes(&self) -> usize {
        self.tasks.len() + self.servers.len()
    }
}

impl Into<FlowGraph> for Graph {
    fn into(self) -> FlowGraph {
        let cluster = VirtualResource::new();
        let source = VirtualResource::new();
        let sink = VirtualResource::new();
        let virtual_ = [&cluster, &sink, &source];

        let mut graph = FlowGraph::new(self.count_nodes() + virtual_.len(), 100);
        
        for task in &self.tasks {
            graph.add_edge(source.get_id(), task.get_id(), 1, 0);
            graph.add_edge(task.get_id(), cluster.get_id(), 1, task.get_request().inner_product() as i64);
        }

        let task_count = self.tasks.len() as i64;
        for server in &self.servers {
            graph.add_edge(cluster.get_id(), server.get_id(), task_count, server.get_request().inner_product() as i64);
            graph.add_edge(server.get_id(), sink.get_id(), task_count, 0);
        }
        graph
    }
}