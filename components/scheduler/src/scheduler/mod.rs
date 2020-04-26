mod resource_profile;
mod scheduler;
mod server;
mod task;
mod virtual_resource;

pub use self::resource_profile::NormalizedResourceProfile;
pub use self::resource_profile::ResourceProfile;
pub type NormalizedTask = Task<NormalizedResourceProfile>;
pub type NormalizedServer = Server<NormalizedResourceProfile>;
pub use self::scheduler::Scheduler;
pub use self::server::Server;
pub use self::task::State;
pub use self::task::Task;
pub use self::task::TaskCommand;
pub use self::virtual_resource::VirtualResource;
use cost_flow::Graphable;

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum Node {
    VirtualResource(VirtualResource),
    Server(NormalizedServer),
    Task(NormalizedTask),
}

impl Graphable for Node {
    fn name_label(&self) -> String {
        match self {
            Node::VirtualResource(t) => t.name_label(),
            Node::Server(t) => t.name_label(),
            Node::Task(t) => t.name_label(),
        }
    }
}
