use super::Displayable;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub struct VirtualResource {
    name: String,
}

impl VirtualResource {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Displayable for VirtualResource {
    fn name(&self) -> String {
        self.name.clone()
    }
}