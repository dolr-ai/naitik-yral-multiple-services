pub mod delegated_identity;
pub mod error;
pub mod ic_agent;
pub mod ic_git;
pub mod msg_builder;
use candid::Principal;
pub use error::*;

use serde::{Deserialize, Serialize};
use web_time::{Duration, SystemTime};

fn current_epoch() -> Duration {
    web_time::SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}

/// A signature, interoperable with ic-agent & yral-identity
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct Signature {
    sig: Option<Vec<u8>>,
    public_key: Option<Vec<u8>>,
    ingress_expiry: Duration,
    delegations: Option<Vec<SignedDelegation>>,
    sender: Principal,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
struct Delegation {
    pub pubkey: Vec<u8>,
    pub expiration_ns: u64,
    pub targets: Option<Vec<Principal>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
struct SignedDelegation {
    pub delegation: Delegation,
    pub signature: Vec<u8>,
}
