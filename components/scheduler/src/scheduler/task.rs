use crate::prelude::*;
use cost_flow::Graphable;
use getset::{Getters, Setters};
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug, Serialize, Eq, Getters, Setters)]
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

impl<T> PartialEq for Task<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
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
                .map(|x| (*x.0, x.1.iter().map(|x| x.normalize(max_profile)).collect()))
                .collect(),
            schedulable: self.schedulable,
        }
    }

    pub fn insert_profile(&mut self, server_id: Uuid, profile: super::ResourceProfile) {
        let entry = self.profiles.entry(server_id).or_insert_with(|| vec![]);
        (*entry).push(profile);
    }
}

impl Task<super::ResourceProfile> {
    pub fn debug_profile(&self) -> super::ResourceProfile {
        self.profiles
            .values()
            .flatten()
            .fold(Default::default(), |acc, x| (acc + *x) / super::ResourceProfile::two())
    }
}

impl Task<super::NormalizedResourceProfile> {
    pub fn profile(&self, server_id: &Uuid) -> Option<super::NormalizedResourceProfile> {
        use num_traits::cast::FromPrimitive;

        let length = self.profiles.get(server_id).map_or(0, Vec::len);
        if let Some(profile) = self.profiles.get(server_id) {
            let sum = profile.iter().enumerate().fold(
                super::NormalizedResourceProfile::default(),
                |acc, (i, x)| {
                    #[allow(clippy::cast_precision_loss)]
                    let i = i as f64;
                    #[allow(clippy::cast_precision_loss)]
                    let length = length as f64;
                    acc + (x.clone() * Decimal::from_f64(1_f64 + i / length).unwrap())
                },
            );
            Some(sum / Decimal::new(length.try_into().unwrap(), 0))
        } else {
            None
        }
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
