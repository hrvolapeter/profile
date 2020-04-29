use crate::import::*;
use cost_flow::Graphable;
use getset::{Getters, Setters};
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug, Serialize, Eq, PartialEq, Getters, Setters)]
pub struct Task<T> {
    #[getset(get = "pub")]
    id: Uuid,
    #[getset(get = "pub", set = "pub")]
    request: Option<T>,
    #[getset(get = "pub")]
    profiles: HashMap<Uuid, Vec<T>>,
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    image: String,
    #[getset(get = "pub")]
    cmd: Option<String>,
    #[getset(get = "pub")]
    realtime: bool,
    /// Signals finished task
    #[getset(get = "pub", set = "pub")]
    schedulable: bool,
}

impl<T> Task<T> {
    pub fn new(
        name: String,
        request: Option<T>,
        image: String,
        realtime: bool,
        cmd: Option<String>,
    ) -> Self {
        Self {
            name,
            cmd,
            request,
            image,
            realtime,
            id: Uuid::new_v4(),
            profiles: Default::default(),
            schedulable: true,
        }
    }
}

impl<T> Hash for Task<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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
            cmd: self.cmd.clone(),
            profiles: self
                .profiles
                .iter()
                .map(|x| (x.0.clone(), x.1.iter().map(|x| x.normalize(max_profile)).collect()))
                .collect(),
            schedulable: self.schedulable,
        }
    }

    pub fn insert_profile(&mut self, server_id: Uuid, profile: super::ResourceProfile) {
        let entry = self.profiles.entry(server_id).or_insert(vec![]);
        (*entry).push(profile);
    }
}

impl Task<super::ResourceProfile> {
    pub fn debug_profile(&self) -> super::ResourceProfile {
        self.profiles
            .values()
            .flatten()
            .fold(Default::default(), |acc, x| (acc + x.clone()) / super::ResourceProfile::two())
    }
}

impl Task<super::NormalizedResourceProfile> {
    pub fn avg_profile(&self, server_id: &Uuid) -> super::NormalizedResourceProfile {
        self.profiles
            .get(server_id)
            .unwrap_or(&vec![])
            .iter()
            .fold(Default::default(), |acc, x| (acc + x.clone()) /  Decimal::new(2,0))
    }
}

impl<T> Graphable for Task<T> {
    fn name_label(&self) -> String {
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
    pub task: Task<super::ResourceProfile>,
    pub state: State,
}
