use crate::{
    codec::{JdwpReadable, JdwpWritable},
    enums::Tag,
    types::{ArrayID, ArrayTypeID},
};

use super::jdwp_command;

/// Creates a new array object of this type with a given length.
#[jdwp_command(4, 1)]
#[derive(Debug, JdwpWritable)]
pub struct NewInstance {
    /// The array type of the new instance
    array_type_id: ArrayTypeID,
    /// The length of the array
    length: i32,
}

#[derive(Debug, JdwpReadable)]
pub struct NewInstanceReply {
    // should always be Tag::Array
    _tag: Tag,
    /// The newly created array object
    pub new_array: ArrayID,
}
