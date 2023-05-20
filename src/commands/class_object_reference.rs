use super::jdwp_command;
use crate::{
    codec::JdwpWritable,
    types::{ClassObjectID, TaggedReferenceTypeID},
};

/// Returns the reference type reflected by this class object.
#[jdwp_command(TaggedReferenceTypeID, 17, 1)]
#[derive(Debug, JdwpWritable)]
pub struct ReflectedType {
    /// The class object
    class_object_id: ClassObjectID,
}
