use crate::import::*;
use cost_flow::Graphable;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub struct VirtualResource {
    name: String,
    id: Uuid,
}

impl VirtualResource {
    pub fn new(name: String) -> Self {
        Self { name, id: Uuid::new_v4() }
    }
}

impl Graphable for VirtualResource {
    fn name_label(&self) -> String {
        self.name.clone()
    }
}
