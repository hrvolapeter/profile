use super::Displayable;
use crate::import::*;
use getset::{Getters, Setters};

#[derive(PartialOrd, PartialEq, Clone, Debug, Serialize, Eq, Ord, Hash, Getters, Setters)]
pub struct Server<T> {
    #[get = "pub"]
    id: Uuid,
    #[getset(get = "pub", set = "pub")]
    hostname: String,
    #[getset(get = "pub", set = "pub")]
    current: Option<T>,
    profiles: Vec<T>,
}

impl<T> Server<T> {
    pub fn new(id: Uuid, hostname: String, current: Option<T>) -> Self {
        Self { hostname, current, id, profiles: vec![] }
    }

    
}

impl Server<super::ResourceProfile> {
    pub fn normalize(&self, max_profile: &super::ResourceProfile) -> super::NormalizedServer {
        super::NormalizedServer {
            current: self.current.map(|x| x.normalize(max_profile)),
            hostname: self.hostname.clone(),
            id: self.id,
            profiles: self.profiles.iter().map(|x| x.normalize(max_profile)).collect(),
        }
    }
}

impl<T> Displayable for Server<T> {
    fn name(&self) -> String {
        self.hostname.clone()
    }
}
