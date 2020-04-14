use super::Node;
use super::Server;
use super::Task;
use super::TaskCommand;
use super::VirtualResource;
use super::NormalizedTask;
use super::NormalizedServer;
use super::ResourceProfile;
use crate::import::*;
use cost_flow;
use cost_flow::{Capacity, Cost};
use futures::channel::mpsc;
use futures_util::sink::SinkExt;
use std::convert::TryInto;
use tokio::sync::watch;
use super::NormalizedResourceProfile;

type Flows = Vec<cost_flow::Edge<Node>>;
type ServerTaskSubscription = mpsc::Sender<TaskCommand>;
type ServerID = Uuid;
pub struct Scheduler {
    tasks: Vec<Task<ResourceProfile>>,
    servers: HashMap<ServerID, Server<ResourceProfile>>,
    server_subscriptions: HashMap<ServerID, ServerTaskSubscription>,
    schedule: HashMap<NormalizedTask, ServerID>,
    notif_channel: (watch::Sender<Flows>, watch::Receiver<Flows>),
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            notif_channel: watch::channel(vec![]),
            tasks: Default::default(),
            servers: Default::default(),
            schedule: Default::default(),
            server_subscriptions: Default::default(),
        }
    }

    pub async fn add_task(&mut self, task: Task<ResourceProfile>) {
        self.tasks.push(task);
        self.schedule().await;
    }

    pub fn get_tasks(&self) -> &Vec<Task<ResourceProfile>> {
        &self.tasks
    }

    /// Add or replace server based on `id`
    pub async fn insert_server(&mut self, server: Server<ResourceProfile>) {
        self.servers.insert(*server.id(), server);
        self.schedule().await;
    }

    pub fn get_servers(&self) -> Vec<&Server<ResourceProfile>> {
        self.servers.values().collect()
    }

    pub fn get_server(&mut self, id: &Uuid) -> Option<&mut Server<ResourceProfile>> {
        self.servers.get_mut(id)
    }

    pub fn subscribe_server(&mut self, id: Uuid, tx: ServerTaskSubscription) {
        self.server_subscriptions.insert(id, tx);
    }

    async fn schedule(&mut self) {
        use cost_flow::MinimumCostFlow;
        let (servers, tasks) = self.normalized();
        let mut graph = build_flow_graph(servers, tasks);
        graph.minimum_cost_flow();
        let paths = graph.paths();
        let _ = self.place_tasks(paths).await;
        let _ = self.notif_channel.0.broadcast(graph.all_edges());
    }

    fn normalized(&self) -> (Vec<NormalizedServer>, Vec<NormalizedTask>) {
        // TODO: 1. check if tasks have higher profile than it's server and update if so
        use std::cmp::max;
        // 2. global maximum profile
        let max_profile = self.servers.values().map(|x| x.current().unwrap_or_else(|| Default::default())).fold(ResourceProfile::default(), |acc, x|
            ResourceProfile {
                ipc:  max(acc.ipc, x.ipc),
                disk: max(acc.disk, x.disk),
                network: max(acc.network, x.network),
                memory: max(acc.memory, x.memory),
            }
        );
        let servers = self.servers.values().map(|x| x.normalize(&max_profile)).collect();

        let tasks = self.tasks.iter().map(|x| x.normalize(&max_profile)).collect();

        (servers, tasks)
    }

    async fn place_tasks(&mut self, paths: Vec<cost_flow::Path<Node>>) -> BoxResult<()> {
        use super::task::State;
        for path in paths {
            let (server, task) = get_server_task(path)?;
            debug!("Scheduling task '{}' on server '{}'", task.name(), server.hostname());
            let old = self.schedule.insert(task.clone(), *server.id());
            if let Some(old) = old {
                let subscription = self.server_subscriptions.get_mut(&old).unwrap();
                let cmd = TaskCommand { task: task.clone(), state: State::Remove };
                subscription.send(cmd).await?;
            }
            // TODO : task cannot have higher resource profile than tha server it's runing on
            let subscription = self.server_subscriptions.get_mut(&server.id()).unwrap();
            let cmd = TaskCommand { task: task.clone(), state: State::Run };
            subscription.send(cmd).await?;
        }

        fn get_server_task(path: cost_flow::Path<Node>) -> BoxResult<(NormalizedServer, NormalizedTask)> {
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

    pub fn subscribe(&self) -> watch::Receiver<Flows> {
        self.notif_channel.1.clone()
    }
}


fn build_flow_graph(servers: Vec<NormalizedServer>, tasks: Vec<NormalizedTask>) -> cost_flow::Graph<Node> {
    let mut graph = cost_flow::Graph::new();
    let cluster =
        graph.add_node(Node::VirtualResource(VirtualResource::new("Cluster".to_string())));
    let task_count = tasks.len();

    for server in servers {
        let cost = if let Some(current) = server.current() {
            current.inner_product()
        } else {
            i64::MAX as u64
        };
        let server = graph.add_node(Node::Server(server.clone()));
        graph.add_edge(
            cluster,
            server,
            Capacity(task_count.try_into().unwrap()),
            Cost(cost.try_into().unwrap()),
        );
        graph.add_edge(server, graph.sink, Capacity(task_count.try_into().unwrap()), Cost(0));
    }

    for task in tasks {
        let cost =
            if let Some(request) = task.request() { request.inner_product() } else { 0 };

        let task = graph.add_node(Node::Task(task.clone()));
        graph.add_edge(graph.source, task, Capacity(1), Cost(0));
        graph.add_edge(task, cluster, Capacity(1), Cost(cost.try_into().unwrap()));

        let unscheduled = graph
            .add_node(Node::VirtualResource(VirtualResource::new("Unscheduled".to_string())));
        graph.add_edge(task, unscheduled, Capacity(1), Cost(0));
        // TODO: add time to running to cost
        graph.add_edge(
            unscheduled,
            graph.sink,
            Capacity(1),
            Cost((NormalizedResourceProfile::MAX.inner_product() * 2).try_into().unwrap()),
        );
    }

    graph
}