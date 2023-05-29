use jdwp_macros::{jdwp_command, JdwpWritable};

use crate::types::ObjectID;

/// Returns the characters contained in the string.
#[jdwp_command(String, 10, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Value {
    /// The String object ID
    string_object: ObjectID,
}
