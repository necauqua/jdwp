use super::jdwp_command;
use crate::{
    codec::{JdwpReadable, JdwpWritable},
    types::{ThreadGroupID, ThreadID},
};

/// Returns the thread group name.
#[jdwp_command(String, 12, 1)]
#[derive(Debug, JdwpWritable)]
pub struct Name {
    /// The thread group object ID
    group: ThreadGroupID,
}

/// Returns the thread group, if any, which contains a given thread group.
#[jdwp_command(Option<ThreadGroupID>, 12, 2)]
#[derive(Debug, JdwpWritable)]
pub struct Parent {
    /// The thread group object ID
    group: ThreadGroupID,
}

/// Returns the live threads and active thread groups directly contained in
/// this thread group.
///
/// Threads and thread groups in child thread groups are not included.
///
/// A thread is alive if it has been started and has not yet been stopped.
/// 
/// See `java.lang.ThreadGroup` for information about active ThreadGroups.
#[jdwp_command(12, 3)]
#[derive(Debug, JdwpWritable)]
pub struct Children {
    /// The thread group object ID
    group: ThreadGroupID,
}

#[derive(Debug, JdwpReadable)]
pub struct ChildrenReply {
    /// Live direct child threads
    pub child_threads: Vec<ThreadID>,
    /// Active child thread groups
    pub child_groups: Vec<ThreadGroupID>,
}
