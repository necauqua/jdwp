use super::jdwp_command;
use crate::{codec::JdwpWritable, types::ObjectID};

/// Returns the characters contained in the string.
#[jdwp_command(String, 10, 1)]
#[derive(Debug, JdwpWritable)]
pub struct Value {
    /// The String object ID
    string_object: ObjectID,
}
