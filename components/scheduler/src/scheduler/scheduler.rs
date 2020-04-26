use super::Node;
use super::NormalizedResourceProfile;
use super::NormalizedServer;
use super::NormalizedTask;
use super::ResourceProfile;
use super::Server;
use super::Task;
use super::TaskCommand;
use super::VirtualResource;
use crate::import::*;
use cost_flow;
use cost_flow::{Capacity, Cost};
use futures::channel::mpsc;
use futures_util::sink::SinkExt;
use tokio::sync::watch;
use rust_decimal::prelude::ToPrimitive;

type Flows = Vec<cost_flow::Edge<Node>>;
type ServerTaskSubscription = mpsc::Sender<TaskCommand>;
type ServerID = Uuid;
type TaskID = Uuid;
pub struct Scheduler {
    tasks: HashMap<TaskID, Task<ResourceProfile>>,
    servers: HashMap<ServerID, Server<ResourceProfile>>,
    server_subscriptions: HashMap<ServerID, ServerTaskSubscription>,
    schedule: HashMap<TaskID, ServerID>,
    notif_channel: (watch::Sender<String>, watch::Receiver<String>),
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            notif_channel: watch::channel(String::new()),
            tasks: Default::default(),
            servers: Default::default(),
            schedule: Default::default(),
            server_subscriptions: Default::default(),
        }
    }

    pub async fn insert_task(&mut self, task: Task<ResourceProfile>) {
        self.tasks.insert(*task.id(), task);
        self.schedule().await;
    }

    pub fn get_tasks(&self) -> Vec<&Task<ResourceProfile>> {
        self.tasks.values().collect()
    }

    pub fn get_task(&mut self, id: &TaskID) -> Option<&mut Task<ResourceProfile>> {
        self.tasks.get_mut(id)
    }

    /// Add or replace server based on `id`
    pub async fn insert_server(&mut self, server: Server<ResourceProfile>) {
        self.servers.insert(*server.id(), server);
        self.schedule().await;
    }

    pub fn get_servers(&self) -> Vec<&Server<ResourceProfile>> {
        self.servers.values().collect()
    }

    pub fn get_server(&mut self, id: &ServerID) -> Option<&mut Server<ResourceProfile>> {
        self.servers.get_mut(id)
    }

    pub fn subscribe_server(&mut self, id: Uuid, tx: ServerTaskSubscription) {
        self.server_subscriptions.insert(id, tx);
    }

    pub async fn schedule(&mut self) {
        use cost_flow::MinimumCostFlow;
        let (servers, tasks) = self.normalized();
        let mut graph = self.build_flow_graph(servers, tasks);
        graph.minimum_cost_flow();
        let paths = graph.paths();
        let _ = self.place_tasks(paths).await;
        let _ = self.notif_channel.0.broadcast(graph.graphviz());
    }

    fn normalized(&self) -> (HashMap<ServerID, NormalizedServer>, HashMap<TaskID, NormalizedTask>) {
        debug!("Normalizing profiles");
        // TODO: 1. check if tasks have higher profile than it's server and update if so
        use std::cmp::max;
        // 2. global maximum profile
        let max_profile = self
            .servers
            .values()
            .map(|x| x.profile().unwrap_or_else(|| ResourceProfile::one()))
            .fold(ResourceProfile::one(), |acc, x| ResourceProfile {
                ipc: max(acc.ipc, x.ipc),
                disk: max(acc.disk, x.disk),
                network: max(acc.network, x.network),
                memory: max(acc.memory, x.memory),
            });
        let servers = self.servers.iter().map(|(k, v)| (k.clone(), v.normalize(&max_profile))).collect();
        let tasks = self.tasks.iter().map(|(k, v)| (k.clone(), v.normalize(&max_profile))).collect();
        (servers, tasks)
    }

    async fn place_tasks(&mut self, paths: Vec<cost_flow::Path<Node>>) -> BoxResult<()> {
        debug!("Assign tasks to servers from graph");
        use super::task::State;
        for path in paths {
            let (server, task) = get_server_task(path)?;
            // 1. Schedule task on assigned server
            let old = self.schedule.insert(*task.id(), *server.id());
            // 2. Check if have been scheduler before
            if let Some(old) = old {
                // 2.1 If assigned to a different server than before deschedule from previous
                // run on a new server
                if old != *server.id() {
                   self.schedule_task(&old, task.clone(), State::Remove).await;
                   debug!("Moving task '{}' from server '{}' to server '{}'", task.name(), old, server.hostname());
                   self.schedule_task(server.id(), task.clone(), State::Run).await;
                }
            } else {
                // 2.2 If haven't scheduler before schedule
                debug!("Scheduling task '{}' on server '{}'", task.name(), server.hostname());
                self.schedule_task(server.id(), task.clone(), State::Run).await;
            }
        }

        fn get_server_task(
            path: cost_flow::Path<Node>,
        ) -> BoxResult<(NormalizedServer, NormalizedTask)> {
            let mut server = None;
            let mut tasks = None;
            for edge in path.edges {
                match edge.source {
                    cost_flow::Node::Node(Node::Server(s)) => server = Some(s.clone()),
                    cost_flow::Node::Node(Node::Task(s)) => tasks = Some(s.clone()),
                    _ => {}
                }
            }
            Ok((server.ok_or("no server")?, tasks.ok_or("no task")?))
        }

        Ok(())
    }

    async fn schedule_task(&mut self, server: &ServerID, task: NormalizedTask, state: super::task::State) {
        let subscription = self.server_subscriptions.get_mut(server).unwrap();
        let cmd = TaskCommand { task, state };
        subscription.send(cmd).await.unwrap();
    }

    pub fn subscribe(&self) -> watch::Receiver<String> {
        self.notif_channel.1.clone()
    }

    fn build_flow_graph(
        &self,
        servers: HashMap<ServerID,NormalizedServer>,
        tasks: HashMap<TaskID, NormalizedTask>
    ) -> cost_flow::Graph<Node> {
        debug!("Building graph");
        let mut graph = cost_flow::Graph::new();
        let cluster =
            graph.add_node(Node::VirtualResource(VirtualResource::new("Cluster".to_string())));
        let task_count = tasks.len();
        let mut servers_node = HashMap::new();
    
        let mut server_usage = HashMap::new();
        for (key, value) in &self.schedule {
            let val = server_usage.entry(value).or_insert(Default::default());
            *val += tasks[key].avg_profile(value).inner_product();
        }
    
        for server in servers.values() {
            // 1. get profile based on benchmark
            let cost = if let Some(cost) = server.profile().as_ref().map(|x| x.inner_product()) {
                 // 2.1 Get server usage, if server unused (not found) 0
                let server_usage = server_usage.get(server.id()).map(|x: &Decimal| x.clone()).unwrap_or_else(|| Default::default());
                // 2.2 (MAX - profile + usage)
                trace!("Server cost: {}: ({} - {} + {}) ", server.hostname(), NormalizedResourceProfile::MAX.inner_product(), cost, server_usage);
                (NormalizedResourceProfile::MAX.inner_product() - cost + server_usage).scaled_i64()
            } else {
                i64::MAX
            };
            debug!("Cost '{}'", cost);
            let server_node = graph.add_node(Node::Server(server.clone()));
            servers_node.insert(server.id().clone(), server_node);
            graph.add_edge(
                cluster,
                server_node,
                Capacity(task_count.try_into().unwrap()),
                Cost(cost),
            );
            graph.add_edge(server_node, graph.sink, Capacity(task_count.try_into().unwrap()), Cost(0));
        }
    
        for task in tasks.values() {
            let cost = if let Some(request) = task.request() { request.inner_product() } else { Decimal::new(0,0) };
    
            let task_node = graph.add_node(Node::Task(task.clone()));
            graph.add_edge(graph.source, task_node, Capacity(1), Cost(0));
            graph.add_edge(task_node, cluster, Capacity(1), Cost(cost.to_i64().unwrap()));
            if let Some(id) = self.schedule.get(task.id()) {
                graph.add_edge(task_node, servers_node[id], Capacity(1), Cost(0));
            }
    
            let unscheduled =
                graph.add_node(Node::VirtualResource(VirtualResource::new("Unscheduled".to_string())));
            graph.add_edge(task_node, unscheduled, Capacity(1), Cost(0));
            graph.add_edge(
                unscheduled,
                graph.sink,
                Capacity(1),
                Cost((NormalizedResourceProfile::MAX.inner_product() * Decimal::new(2,0)).scaled_i64()),
            );
        }
    
        graph
    }
}

trait DecimalConvert {
   fn scaled_i64(&self) -> i64;
}

impl DecimalConvert for Decimal {
    fn scaled_i64(&self) -> i64 {
        (self * Decimal::new(100, 0)).to_i64().unwrap()

    }
}