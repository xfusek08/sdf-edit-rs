use enum_utils::ToIndex;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, ToIndex, Serialize, Deserialize)]
pub enum Operation {
    Add,
    Subtract,
    Intersect,
    // TODO: Paint
}
