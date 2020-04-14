use super::Displayable;
use crate::import::*;
use super::NormalizedTask;
use getset::{Getters, Setters};
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug, Serialize, Eq, PartialEq, Hash, Getters, Setters)]
pub struct Task<T> {
    #[getset(get = "pub")]
    id: Uuid,
    #[getset(get = "pub", set = "pub")]
    request: Option<T>,
    #[getset(get = "pub", set = "pub")]
    profiles: HashMap<Uuid, T>,
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    image: String,
    #[getset(get = "pub")]
    realtime: bool,
}

impl<T> Task<T> {
    pub fn new(
        name: String,
        request: Option<T>,
        image: String,
        realtime: bool,
    ) -> Self {
        Self { name, request, image, realtime, id: Uuid::new_v4(), profiles: Default::default() }
    }
}

impl<T> Hash for Task<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash();
    }
}

impl Task<super::ResourceProfile> {
    pub fn normalize(&self, max_profile: &super::ResourceProfile) -> super::NormalizedTask {
        super::NormalizedTask {
            id: self.id,
            image: self.image.clone(),
            realtime: self.realtime,
            name: self.name.clone(),
            request: self.request.map(|x| x.normalize(&max_profile)),
            profiles: self.profiles.iter().map(|x| (x.0, x.1.normalize(max_profile))).collect(),
        }
    }
}

impl<T> Displayable for Task<T> {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub enum State {
    Run,
    Remove,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub struct TaskCommand {
    pub task: NormalizedTask,
    pub state: State,
}
