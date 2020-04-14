use super::Displayable;
use crate::import::*;

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

impl Displayable for VirtualResource {
    fn name(&self) -> String {
        self.name.clone()
    }
}
