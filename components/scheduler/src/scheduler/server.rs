use super::ResourceProfile;
use super::Displayable;
use crate::import::*;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub struct Server {
    pub id: String,
    // max utilization
    pub current: Option<ResourceProfile>,
}

impl Server {
    pub fn new(id: String, current: Option<ResourceProfile>) -> Self {
        Self { id, current }
    }

    pub fn get_current(&self) -> &Option<ResourceProfile> {
        &self.current
    }
}

impl Displayable for Server {
    fn name(&self) -> String {
        self.id.clone()
    }
}