use mcmf::{Capacity, Cost, Flow, GraphBuilder, Vertex, Path};
use tokio::sync::watch;
use futures::channel::mpsc;
use crate::import::*;
use super::Task;
use super::TaskCommand;
use super::Server;
use super::Node;
use super::VirtualResource;
use std::convert::TryInto;
use futures_util::sink::SinkExt;
use std::cmp::max;

type Flows = Vec<Flow<Node>>;
type ServerTaskSubscription = mpsc::Sender<TaskCommand>;
type ServerID = String;
pub struct Scheduler {
    tasks: Vec<Task>,
    servers: HashMap<ServerID, Server>,
    server_subscriptions: HashMap<ServerID, ServerTaskSubscription>,
    schedule: HashMap<Task, ServerID>,
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

    pub async fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.schedule().await;
    }

    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    /// Add or replace server based on `id`
    pub async fn insert_server(&mut self, server: Server) {
        self.servers.insert(server.id.clone(), server);
        self.schedule().await;
    }

    pub fn get_servers(&self) -> Vec<&Server> {
        self.servers.values().collect()
    }

    pub fn subscribe_server(&mut self, id: String, tx: ServerTaskSubscription) {
        self.server_subscriptions.insert(id, tx);
    }

    async fn schedule(&mut self) {
        let graph = self.build_flow_graph();
        let (_, paths, flows) = graph.mcmf();
        let _ = self.place_tasks(paths).await;
        let _ = self.notif_channel.0.broadcast(flows);
    }

    async fn place_tasks(&mut self, paths: Vec<Path<Node>>) -> BoxResult<()> {
        use super::task::State;
        for path in paths {
            let (server, task) = get_server_task(path)?;
            let old = self.schedule.insert(task.clone(), server.id.clone());
            if let Some(old) = old {
                let subscription = self.server_subscriptions.get_mut(&old).unwrap();
                let cmd = TaskCommand {
                    task: task.clone(),
                    state: State::Remove,
                };
                subscription.send(cmd).await?;
            }
            let subscription = self.server_subscriptions.get_mut(&server.id).unwrap();
            let cmd = TaskCommand {
                task: task.clone(),
                state: State::Run,
            };
            subscription.send(cmd).await?;
        }

        fn get_server_task(path: Path<Node>) -> BoxResult<(Server, Task)> {
            let mut server = None;
            let mut tasks = None;
            for vertex in path.vertices() {
                match vertex {
                    Vertex::Node(Node::Server(s)) => server = Some(s.clone()),
                    Vertex::Node(Node::Task(s)) => tasks = Some(s.clone()),
                    _ => {},
                }
            }
            Ok((server.ok_or("no server")?, tasks.ok_or("no task")?))
        }

        Ok(())
    }

    fn build_flow_graph(&self) -> GraphBuilder<Node> {
        let mut graph = GraphBuilder::new();
        let cluster = Node::VirtualResource(VirtualResource::new("Cluster".to_string()));
        let task_count = self.tasks.len();

        let mut strongest_machine = &Default::default();
        for server in self.servers.values() {
            let cost = if let Some(current) = server.get_current() {
                strongest_machine = max(strongest_machine, current);
                current.inner_product()
            } else {
                i64::MAX as u64
            };

            graph.add_edge(
                cluster.clone(),
                Node::Server(server.clone()),
                Capacity(task_count as i64),
                Cost(cost.try_into().unwrap()),
            );
            graph.add_edge(
                Node::Server(server.clone()),
                Vertex::Sink,
                Capacity(task_count as i64),
                Cost(0),
            );
        }

        for task in &self.tasks {
            let cost = if let Some(request) = task.get_request() {
                request.inner_product()
            } else {
                0
            };

            graph.add_edge(Vertex::Source, Node::Task(task.clone()), Capacity(1), Cost(0));
            graph.add_edge(
                Node::Task(task.clone()),
                cluster.clone(),
                Capacity(1),
                Cost(cost.try_into().unwrap()),
            );

            let unscheduled = Node::VirtualResource(VirtualResource::new("Unscheduled".to_string()));
            graph.add_edge(Node::Task(task.clone()), unscheduled.clone(), Capacity(1), Cost(0));
            // TODO: add time to running to cost
            graph.add_edge(unscheduled, Vertex::Sink, Capacity(1), Cost((strongest_machine.inner_product() * 2) as i64));
        }

        graph
    }

    pub fn subscribe(&self) -> watch::Receiver<Flows> {
        self.notif_channel.1.clone()
    }
}
