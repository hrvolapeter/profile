use super::Node;
use super::NormalizedResourceProfile;
use super::NormalizedServer;
use super::NormalizedTask;
use super::ResourceProfile;
use super::Server;
use super::Task;
use super::TaskCommand;
use super::VirtualResource;
use crate::prelude::*;
use cost_flow::{Capacity, Cost};
use futures::channel::mpsc;
use futures_util::sink::SinkExt;
use tokio::sync::watch;
use rust_decimal::prelude::ToPrimitive;

type ServerTaskSubscription = mpsc::Sender<TaskCommand>;
type ServerID = Uuid;
type TaskID = Uuid;
pub struct Scheduler {
    tasks: HashMap<TaskID, Task<ResourceProfile>>,
    servers: HashMap<ServerID, Server<ResourceProfile>>,
    // Channel to agent running on server
    server_subscriptions: HashMap<ServerID, ServerTaskSubscription>,
    schedule: HashMap<TaskID, ServerID>,
    // Channel for updating web ui
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

    /// Check if task was scheduler before, if so and it's finished running make it schedulable
    /// else create a new task.
    /// 
    /// Schedulability property is based on the task name
    pub async fn insert_task(&mut self, task: Task<ResourceProfile>) {
        if let Some(task) = self.tasks.values_mut().find(|x| x.name() == task.name()) {
            task.set_schedulable(true);
        } else {
            self.tasks.insert(*task.id(), task);
        }
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

    /// Runs scheduling pipeline
    /// 1. computes flow graph
    /// 2. creates new schedule
    /// 3. assign tasks to server based on schedule (agent are notified of the change)
    pub async fn schedule(&mut self) {
        use cost_flow::MinimumCostFlow;
        let (servers, tasks) = self.normalize();
        let mut graph = self.build_flow_graph(&servers, &tasks);
        graph.minimum_cost_flow();
        let paths = graph.paths();
        let _ = self.place_tasks(paths).await;
        let _ = self.notif_channel.0.broadcast(graph.graphviz());
    }

    /// Finds maximum profile for all servers and uses the most performant server as a maximum value
    /// Each resource is normalized to value between (0, 1), the max profile beeing all 1
    fn normalize(&self) -> (HashMap<ServerID, NormalizedServer>, HashMap<TaskID, NormalizedTask>) {
        use std::cmp::max;

        debug!("Normalizing profiles");
        // TODO: 1. check if tasks have higher profile than it's server and update if so
        // 2. global maximum profile
        let max_profile = self
            .servers
            .values()
            .map(|x| x.profile().unwrap_or_else(|| ResourceProfile::ONE))
            .fold(ResourceProfile::default(), |acc, x| ResourceProfile {
                ipc: max(acc.ipc, x.ipc),
                disk: max(acc.disk, x.disk),
                network: max(acc.network, x.network),
                memory: max(acc.memory, x.memory),
            });
        let servers = self.servers.iter().map(|(k, v)| (*k, v.normalize(&max_profile))).collect();
        let tasks = self.tasks.iter().map(|(k, v)| (*k, v.normalize(&max_profile))).collect();
        (servers, tasks)
    }

    /// Assign task to a server based on result from flow graph
    async fn place_tasks(&mut self, paths: Vec<cost_flow::Path<Node>>) -> BoxResult<()> {
        use super::task::State;

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

        debug!("Assign tasks to servers from graph");
        let mut normalized_tasks = HashMap::new();

        // 1. Start with empty schedule
        let old = self.schedule.clone();
        self.schedule = HashMap::new();
        for path in paths {
            let (server, task) = get_server_task(path)?;
            // 2. Assign task to a server
            self.schedule.insert(*task.id(), *server.id());
            normalized_tasks.insert(task.id().clone(), task);
        }
        // 3. Get tasks that didn't run before or have been moved to different server
        let mut to_schedule = self.schedule.clone();
        to_schedule.retain(|k,v| !(old.get(k).is_some() && old[k] == *v));

        for (task_id, server_id) in to_schedule {
            let task = self.tasks[&task_id].clone();
            debug!("Scheduling task '{}' on server '{}'", task.name(), self.servers[&server_id].hostname());
            self.schedule_task(&server_id, task, State::Run).await;
        }

        // 4. Get descheduled tasks or previous task allocation that has been moved
        let mut to_deschedule = old.clone();
        to_deschedule.retain(|k,v| self.schedule.get(k).is_none() || self.schedule[k] != *v);

        for (task_id, server_id) in to_deschedule {
            let task = self.tasks[&task_id].clone();
            debug!("Descheduling task '{}' from server '{}'", task.name(), server_id);
            self.schedule_task(&server_id, task, State::Remove).await;
        }

        Ok(())
    }

    async fn schedule_task(&mut self, server: &ServerID, task: super::Task<super::ResourceProfile>, state: super::task::State) {
        let subscription = self.server_subscriptions.get_mut(server).unwrap();
        let cmd = TaskCommand { task, state };
        subscription.send(cmd).await.unwrap();
    }

    pub fn subscribe(&self) -> watch::Receiver<String> {
        self.notif_channel.1.clone()
    }

    fn build_flow_graph(
        &self,
        servers: &HashMap<ServerID,NormalizedServer>,
        tasks: &HashMap<TaskID, NormalizedTask>
    ) -> cost_flow::Graph<Node> {
        debug!("Building graph");
        let mut graph = cost_flow::Graph::new();
        let cluster =
            graph.add_node(Node::VirtualResource(VirtualResource::new("Cluster".to_string())));
        let task_count = tasks.len();
    
        // 1. Get current server utilization
        let mut server_usage = HashMap::new();
        for (key, value) in &self.schedule {
            let val = server_usage.entry(*value).or_insert_with(Default::default);
            *val += tasks[key].profile(value).unwrap_or(NormalizedResourceProfile::default());
        }

        let mut server_to_node = HashMap::new();
    
        // 2. Add servers to flow graph
        for server in servers.values() {
            let node = graph.add_node(Node::Server(server.clone()));

            // 2.1. get profile based on benchmark
            let cost = if let Some(profile) = server.profile().as_ref() {
                 // 2.2 Get server usage, if server unused (not found) 0
                let server_usage = server_usage.get(server.id()).map_or_else(Default::default, |x: &NormalizedResourceProfile| x.clone());
                // 2.3 (MAX - profile + usage)
                trace!("Server cost: {}: ({} - {:?} (cost {}) + {:?}(cost {})) ", server.hostname(), NormalizedResourceProfile::MAX.inner_product(), profile, profile.inner_product(), server_usage, server_usage.inner_product());
                server_to_node.insert(server.id(), (profile.clone() - server_usage.clone(), node));
                (NormalizedResourceProfile::MAX.inner_product() - profile.inner_product() + server_usage.inner_product()).scaled_i64()
            } else {
                i64::MAX
            };
            trace!("Cost result {}", cost);
            graph.add_edge(
                cluster,
                node,
                Capacity(task_count.try_into().unwrap()),
                Cost(cost),
            );
            graph.add_edge(node, graph.sink, Capacity(task_count.try_into().unwrap()), Cost(0));
        }
    
        // 3. Add tasks to flow graph
        for task in tasks.values() {
            // 3.1 Continue if task is finished running
            if !task.schedulable() {
                continue;
            }
            let cost = if let Some(server_id) = task.profiles().keys().next() {
                // TODO: server can produce different profiles for the same task/job
                // even though resource distribution is simillar this can result if having different
                // cost when scheduling task
                task.profile(server_id).map_or(0, |x| x.inner_product().to_i64().unwrap())
            } else {
                0
            };

            // 3.2 Create task and connect to source
            let task_node = graph.add_node(Node::Task(task.clone()));
            graph.add_edge(graph.source, task_node, Capacity(1), Cost(0));
    
            // 3.2 Connect task with servers
            if let Some(request) = task.request() {
                 // 3.2.1 Connect task with servers that meet requirements
                 for server in servers.values() {
                    let (free_resources, server_node) = &server_to_node[server.id()];
                    let diff = free_resources.clone() - request.clone();
                    if diff.has_negative_resource() {
                        continue;
                    }
                    graph.add_edge(task_node, *server_node, Capacity(1), Cost(cost));
                }
            } else {
                 // 3.2.2 Connect task with cluster node if no minimal requirements
                 graph.add_edge(task_node, cluster, Capacity(1), Cost(cost.to_i64().unwrap()));
                 if let Some(id) = self.schedule.get(task.id()) {
                     graph.add_edge(task_node, server_to_node[id].1, Capacity(1), Cost(0));
                 }
            }
            
    
            // 3.3 Allow tasks to remain unscheduled
            let unscheduled =
                graph.add_node(Node::VirtualResource(VirtualResource::new(format!("Unscheduled {}", task.name()))));
            graph.add_edge(task_node, unscheduled, Capacity(1), Cost(0));
            graph.add_edge(
                unscheduled,
                graph.sink,
                Capacity(1),
                Cost((NormalizedResourceProfile::MAX.inner_product()).scaled_i64()),
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