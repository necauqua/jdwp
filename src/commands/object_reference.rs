use super::jdwp_command;
use crate::{
    codec::JdwpWritable,
    types::{ObjectID, TaggedReferenceTypeID},
};

#[jdwp_command(TaggedReferenceTypeID, 9, 1)]
#[derive(Debug, JdwpWritable)]
pub struct ReferenceType {
    /// The object ID
    object: ObjectID,
}
