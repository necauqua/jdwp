use crate::{
    codec::JdwpWritable,
    types::{ArrayID, ArrayRegion, Untagged},
};

use super::jdwp_command;

/// Returns the number of components in a given array.
#[jdwp_command(i32, 13, 1)]
#[derive(Debug, JdwpWritable)]
pub struct Length {
    /// The array object ID
    array_id: ArrayID,
}

/// Returns a range of array components.
/// The specified range must be within the bounds of the array.
#[jdwp_command(ArrayRegion, 13, 2)]
#[derive(Debug, JdwpWritable)]
pub struct GetValues {
    /// The array object ID
    array_id: ArrayID,
    /// The first index to retrieve
    first_index: i32,
    /// The number of components to retrieve
    length: i32,
}

/// Sets a range of array components.
/// The specified range must be within the bounds of the array.
/// For primitive values, each value's type must match the array component type
/// exactly.
/// For object values, there must be a widening reference conversion from the
/// value's type to the array component type and the array component type must
/// be loaded.
#[jdwp_command((), 13, 3)]
#[derive(Debug, JdwpWritable)]
pub struct SetValues {
    /// The array object ID
    array_id: ArrayID,
    /// The first index to set
    first_index: i32,
    /// Values to set
    values: Vec<Untagged>,
}
