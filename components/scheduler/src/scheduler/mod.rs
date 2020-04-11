mod resource_profile;
mod scheduler;
mod server;
mod task;
mod virtual_resource;

pub use self::resource_profile::ResourceProfile;
pub use self::server::Server;
pub use self::task::Task;
pub use self::virtual_resource::VirtualResource;
pub use self::scheduler::Scheduler;
pub use self::task::TaskCommand;
pub use self::task::State;


pub trait Displayable {
    fn name(&self) -> String;
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

