use serde::{Deserialize, Serialize};

// By default, player is Red, and computer is Black.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) enum Side {
    Red,
    Black,
}
