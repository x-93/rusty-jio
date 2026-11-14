// subnets.rs
//
// Subnetwork IDs in rusty-jio.
// In BlockDAG architectures, transactions can belong to subnets (e.g. Native currency, Smart Contracts).

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const SUBNETWORK_ID_SIZE: usize = 20;

#[derive(
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct SubnetworkId(pub [u8; SUBNETWORK_ID_SIZE]);

impl SubnetworkId {
    pub const NATIVE: SubnetworkId = SubnetworkId([0u8; SUBNETWORK_ID_SIZE]);
    pub const COINBASE: SubnetworkId =
        SubnetworkId([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; SUBNETWORK_ID_SIZE]) -> Self {
        Self(bytes)
    }

    #[inline(always)]
    pub fn is_native(&self) -> bool {
        *self == Self::NATIVE
    }

    #[inline(always)]
    pub fn is_coinbase(&self) -> bool {
        *self == Self::COINBASE
    }
}

impl Display for SubnetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut hex = [0u8; SUBNETWORK_ID_SIZE * 2];
        faster_hex::hex_encode(&self.0, &mut hex).unwrap();
        f.write_str(std::str::from_utf8(&hex).unwrap())
    }
}
