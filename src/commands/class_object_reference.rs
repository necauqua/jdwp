use jdwp_macros::{jdwp_command, JdwpWritable};

use crate::types::{ClassObjectID, TaggedReferenceTypeID};

/// Returns the reference type reflected by this class object.
#[jdwp_command(TaggedReferenceTypeID, 17, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ReflectedType {
    /// The class object
    class_object_id: ClassObjectID,
}
