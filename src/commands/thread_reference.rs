use jdwp_macros::jdwp_command;

use crate::{codec::JdwpWritable, types::ThreadID};

/// Returns the thread name.
#[jdwp_command(String, 11, 1)]
#[derive(Debug, JdwpWritable)]
pub struct Name {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Suspends the thread.
///
/// Unlike java.lang.Thread.suspend(), suspends of both the virtual machine and
/// individual threads are counted. Before a thread will run again, it must be
/// resumed the same number of times it has been suspended.
///
/// Suspending single threads with command has the same dangers
/// java.lang.Thread.suspend(). If the suspended thread holds a monitor needed
/// by another running thread, deadlock is possible in the target VM (at least
/// until the suspended thread is resumed again).
///
/// The suspended thread is guaranteed to remain suspended until resumed through
/// one of the JDI resume methods mentioned above; the application in the target
/// VM cannot resume the suspended thread through {@link
/// java.lang.Thread#resume}.
///
/// Note that this doesn't change the status of the thread (see the ThreadStatus
/// command.) For example, if it was Running, it will still appear running to
/// other threads.
#[jdwp_command((), 11, 2)]
#[derive(Debug, JdwpWritable)]
pub struct Suspend {
    /// The thread object ID.
    pub thread: ThreadID,
}
