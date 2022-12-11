
use enum_utils::ToIndex;

#[derive(Clone, PartialEq, Debug, ToIndex)]
pub enum Operation {
    Add,
    Subtract,
    Intersect,
    // TODO: Paint
}
