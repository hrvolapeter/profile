use crate::prelude::*;
use cost_flow::Graphable;
use getset::{Getters, Setters};

#[derive(PartialOrd, PartialEq, Clone, Debug, Serialize, Eq, Ord, Hash, Getters, Setters)]
pub struct Server<T> {
    #[get = "pub"]
    id: Uuid,
    #[getset(get = "pub", set = "pub")]
    hostname: String,
    #[getset(get = "pub", set = "pub")]
    profile: Option<T>,
}

impl<T> Server<T> {
    pub fn new(id: Uuid, hostname: String, profile: Option<T>) -> Self {
        Self { hostname, profile, id }
    }
}

impl Server<super::ResourceProfile> {
    pub fn normalize(&self, max_profile: &super::ResourceProfile) -> super::NormalizedServer {
        super::NormalizedServer {
            profile: self.profile.map(|x| x.normalize(max_profile)),
            hostname: self.hostname.clone(),
            id: self.id,
        }
    }
}

impl<T> Graphable for Server<T> {
    fn name_label(&self) -> String {
        self.hostname.clone()
    }
}
