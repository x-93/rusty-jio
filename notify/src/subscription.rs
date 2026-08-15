use crate::scope::Scope;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Subscribe(Scope),
    Unsubscribe(Scope),
}
