use crate::prelude::*;
use derive_more::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign};
use std::cmp::Ordering;

#[derive(
    Default, Copy, Clone, PartialEq, Hash, Eq, Debug, Serialize, Add, AddAssign, Sub, SubAssign,
)]
pub struct ResourceProfile {
    pub ipc: Decimal,
    pub memory: u64,
    pub network: u64,
    pub disk: u64,
}

impl ResourceProfile {
    pub const ONE: ResourceProfile = Self { ipc: one(), memory: 1, network: 1, disk: 1 };

    pub fn two() -> ResourceProfile {
        Self::ONE + Self::ONE
    }

    pub fn normalize(&self, other: &ResourceProfile) -> NormalizedResourceProfile {
        NormalizedResourceProfile {
            ipc: self.ipc.normalize_to(&other.ipc),
            memory: self.memory.normalize_to(&other.memory),
            network: self.network.normalize_to(&other.network),
            disk: self.disk.normalize_to(&other.disk),
        }
    }
}

impl std::ops::Div for ResourceProfile {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self::Output {
        self.ipc /= rhs.ipc;
        self.memory /= rhs.memory;
        self.network /= rhs.network;
        self.disk /= rhs.disk;
        self
    }
}

#[derive(
    Default,
    Clone,
    PartialEq,
    Hash,
    Eq,
    Debug,
    Serialize,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Div,
    Mul,
    MulAssign,
)]
pub struct NormalizedResourceProfile {
    ipc: Decimal,
    memory: Decimal,
    network: Decimal,
    disk: Decimal,
}

const fn one() -> Decimal {
    Decimal::from_parts(1, 0, 0, false, 0)
}

const fn two() -> Decimal {
    Decimal::from_parts(2, 0, 0, false, 0)
}

impl NormalizedResourceProfile {
    pub const MAX: NormalizedResourceProfile =
        NormalizedResourceProfile { ipc: one(), disk: one(), memory: one(), network: one() };

    pub fn inner_product(&self) -> Decimal {
        self.ipc  * two() + self.memory * two() + self.network + self.disk
    }

    pub fn has_negative_resource(&self) -> bool {
        self.ipc.is_sign_negative() || self.memory.is_sign_negative() || self.network.is_sign_negative() || self.disk.is_sign_negative()
    }
}

impl Ord for NormalizedResourceProfile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner_product().cmp(&other.inner_product())
    }
}

impl PartialOrd for NormalizedResourceProfile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn one() {
        assert_eq!(one(), Decimal::new(1, 0));
    }
}

trait DecimalNormalize {
    fn normalize_to(&self, other: &Self) -> Decimal;
}

impl DecimalNormalize for Decimal {
    fn normalize_to(&self, other: &Decimal) -> Decimal {
        // TODO: task can have higher profile than the server it running one
        // Server profile should be updated if task has better performance than benchmark
        // debug_assert!(self < other);
        self / other
    }
}

impl DecimalNormalize for u64 {
    fn normalize_to(&self, other: &u64) -> Decimal {
        let num: Decimal = (*self).into();
        let other: Decimal = (*other).into();
        num.normalize_to(&other)
    }
}
