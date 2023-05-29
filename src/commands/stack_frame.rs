use crate::{
    codec::JdwpWritable,
    enums::Tag,
    types::{FrameID, TaggedObjectID, ThreadID, Value},
};

use super::jdwp_command;

/// Returns the value of one or more local variables in a given frame.
///
/// Each variable must be visible at the frame's code index.
///
/// Even if local variable information is not available, values can be retrieved
/// if the front-end is able to determine the correct local variable index.
/// (Typically, this index can be determined for method arguments from the
/// method signature without access to the local variable table information.)
#[jdwp_command(Vec<Value>, 16, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct GetValues {
    /// The frame's thread.
    pub thread_id: ThreadID,
    /// The frame ID.
    pub frame_id: FrameID,
    /// Local variable indices and types to get.
    pub slots: Vec<(u32, Tag)>,
}

/// Sets the value of one or more local variables.
///
/// Each variable must be visible at the current frame code index. For primitive
/// values, the value's type must match the variable's type exactly. For object
/// values, there must be a widening reference conversion from the value's type
/// to thevariable's type and the variable's type must be loaded.
///
/// Even if local variable information is not available, values can be set, if
/// the front-end is able to determine the correct local variable index.
/// (Typically, thisindex can be determined for method arguments from the method
/// signature without access to the local variable table information.)
#[jdwp_command((), 16, 2)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct SetValues {
    /// The frame's thread.
    pub thread_id: ThreadID,
    /// The frame ID.
    pub frame_id: FrameID,
    /// Local variable indices and values to set.
    pub slots: Vec<(u32, Value)>,
}

/// Returns the value of the 'this' reference for this frame.
///
/// If the frame's method is static or native, the reply will contain the null
/// object reference.
#[jdwp_command(Option<TaggedObjectID>, 16, 3)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ThisObject {
    /// The frame's thread.
    pub thread_id: ThreadID,
    /// The frame ID.
    pub frame_id: FrameID,
}

/// Pop the top-most stack frames of the thread stack, up to, and including
/// 'frame'. The thread must be suspended to perform this command. The top-most
/// stack frames are discarded and the stack frame previous to 'frame' becomes
/// the current frame. The operand stack is restored -- the argument values are
/// added back and if the invoke was not invokestatic, objectref is added back
/// as well. The Java virtual machine program counter is restored to the opcode
/// of the invoke instruction.
///
/// Since JDWP version 1.4.
///
/// Requires `canPopFrames` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[jdwp_command((), 16, 4)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct PopFrames {
    /// The frame's thread.
    pub thread_id: ThreadID,
    /// The frame ID.
    pub frame_id: FrameID,
}
