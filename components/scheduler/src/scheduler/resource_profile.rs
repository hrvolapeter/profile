use serde::Serialize;
use std::cmp::Ordering;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize)]
pub struct ResourceProfile {
    pub cpu: u64,
    pub memory: u64,
    pub network: u64,
    pub disk: u64,
}

impl ResourceProfile {
    pub fn inner_product(&self) -> u64 {
        self.cpu + self.memory + self.network + self.disk
    }
}

impl Add for ResourceProfile {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            cpu: self.cpu + other.cpu,
            memory: self.memory + other.memory,
            network: self.network + other.network,
            disk: self.disk + other.disk,
        }
    }
}

impl AddAssign for ResourceProfile {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl Sub for ResourceProfile {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            cpu: self.cpu - other.cpu,
            memory: self.memory - other.memory,
            network: self.network - other.network,
            disk: self.disk - other.disk,
        }
    }
}

impl SubAssign for ResourceProfile {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl PartialOrd for ResourceProfile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceProfile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner_product().cmp(&other.inner_product())
    }
}