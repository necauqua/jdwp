use jdwp_macros::JdwpReadable;
use std::ops::Deref;

use super::jdwp_command;

use crate::{
    codec::JdwpWritable,
    enums::Tag,
    types::{ArrayID, ArrayTypeID},
};

/// Creates a new array object of this type with a given length.
#[jdwp_command(4, 1)]
#[derive(Debug, Clone, JdwpWritable)]
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

impl Deref for NewInstanceReply {
    type Target = ArrayID;

    fn deref(&self) -> &Self::Target {
        &self.new_array
    }
}
