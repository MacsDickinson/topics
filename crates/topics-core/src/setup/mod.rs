//! Canonical human setup/review API. These types are never model authority.
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
fn required<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    T::deserialize(d)
}
macro_rules! value {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(#[serde(deserialize_with="required")] pub $field: $ty),* }
    };
}
macro_rules! choices {
    ($name:ident { $($variant:ident),* }) => {
        #[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all="snake_case")]
        pub enum $name { $($variant),* }
    };
}

mod inventory;
mod jobs;
mod review;
pub use inventory::*;
pub use jobs::*;
pub use review::*;

#[cfg(test)]
mod tests;
