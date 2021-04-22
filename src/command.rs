use std::cmp::{Eq, PartialEq};

/// Port <-> Starboard
/// ^ Ahead
/// v asterN
/// simple use ' ' for stop engine
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Port,
    Starboard,
    Ahead,
    Astern,
    SE,
}
