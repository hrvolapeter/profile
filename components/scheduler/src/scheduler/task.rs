use super::ResourceProfile;
use super::Displayable;
use crate::import::*;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub struct Task {
    pub request: Option<ResourceProfile>,
    pub name: String,
    pub image: String,
    pub realtime: bool
}

impl Task {
    pub fn new(name: String, request: Option<ResourceProfile>, image: String, realtime: bool) -> Self {
        Self { name, request, image, realtime }
    }

    pub fn get_request(&self) -> &Option<ResourceProfile> {
        &self.request
    }

    pub fn get_image(&self) -> &String {
        &self.image
    }

    pub fn is_realtime(&self)-> bool {
        self.realtime
    }
}

impl Displayable for Task {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub enum State {
    Run,
    Remove,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub struct TaskCommand {
    pub task: Task,
    pub state: State,
}