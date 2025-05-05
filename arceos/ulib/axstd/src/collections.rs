// axstd/src/collections.rs
pub use alloc::collections::*;

use hashbrown::hash_map as base;

pub use base::{HashMap, Entry, OccupiedEntry, VacantEntry};
