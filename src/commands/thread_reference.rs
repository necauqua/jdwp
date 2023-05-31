use jdwp_macros::jdwp_command;

use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    enums::{SuspendStatus, ThreadStatus},
    types::{FrameID, Location, TaggedObjectID, ThreadGroupID, ThreadID, Value},
};

/// Returns the thread name.
#[jdwp_command(String, 11, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Name {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Suspends the thread.
///
/// Unlike `java.lang.Thread.suspend()`, suspends of both the virtual machine
/// and individual threads are counted. Before a thread will run again, it must
/// be resumed the same number of times it has been suspended.
///
/// Suspending single threads with command has the same dangers
/// `java.lang.Thread.suspend()`. If the suspended thread holds a monitor needed
/// by another running thread, deadlock is possible in the target VM (at least
/// until the suspended thread is resumed again).
///
/// The suspended thread is guaranteed to remain suspended until resumed through
/// one of the JDI resume methods mentioned above; the application in the target
/// VM cannot resume the suspended thread through `java.lang.Thread.resume()`.
///
/// Note that this doesn't change the status of the thread (see the
/// [ThreadStatus] command.) For example, if it was Running, it will still
/// appear running to other threads.
#[jdwp_command((), 11, 2)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Suspend {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Resumes the execution of a given thread.
///
/// If this thread was not previously suspended by the front-end, calling this
/// command has no effect. Otherwise, the count of pending suspends on this
/// thread is decremented. If it is decremented to 0, the thread will continue
/// to execute.
#[jdwp_command((), 11, 3)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Resume {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Returns the current status of a thread.
///
/// The thread status reply indicates the thread status the last time it was
/// running. the suspend status provides information on the thread's suspension,
/// if any.
#[jdwp_command((ThreadStatus, SuspendStatus), 11, 4)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Status {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Returns the thread group that contains a given thread.
#[jdwp_command(ThreadGroupID, 11, 5)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ThreadGroup {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Returns the current call stack of a suspended thread.
///
/// The sequence of frames starts with the currently executing frame, followed
/// by its caller, and so on. The thread must be suspended, and the returned
/// frameID is valid only while the thread is suspended.
#[jdwp_command(Vec<(FrameID, Location)>, 11, 6)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Frames {
    /// The thread object ID.
    pub thread: ThreadID,
    /// The index of the first frame to retrieve.
    pub start_frame: u32,
    /// The amount of frames to retrieve.
    pub limit: FrameLimit,
}

/// A nice readable enum to be used in place of raw `i32` with a special meaning
/// for -1.
#[derive(Debug, Clone)]
pub enum FrameLimit {
    Limit(u32),
    AllRemaining,
}

impl JdwpWritable for FrameLimit {
    fn write<W: std::io::Write>(&self, write: &mut JdwpWriter<W>) -> std::io::Result<()> {
        match self {
            FrameLimit::Limit(n) => n.write(write),
            FrameLimit::AllRemaining => (-1i32).write(write),
        }
    }
}

/// Returns the count of frames on this thread's stack.
///
/// The thread must be suspended, and the returned count is valid only while the
/// thread is suspended.
///
/// Returns [ThreadNotSuspended](crate::enums::ErrorCode::ThreadNotSuspended) if
/// not suspended.
#[jdwp_command(u32, 11, 7)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct FrameCount {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Returns the objects whose monitors have been entered by this thread.
///
/// The thread must be suspended, and the returned information is relevant only
/// while the thread is suspended.
///
/// Requires `can_get_owned_monitor_info` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[jdwp_command(Vec<TaggedObjectID>, 11, 8)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct OwnedMonitors {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Returns the object, if any, for which this thread is waiting.
///
/// The thread may be waiting to enter a monitor, or it may be waiting, via the
/// `java.lang.Object.wait` method, for another thread to invoke the notify
/// method. The thread must be suspended, and the returned information is
/// relevant only while the thread is suspended.
///
/// Requires `can_get_current_contended_monitor` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[jdwp_command(Option<TaggedObjectID>, 11, 9)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct CurrentContendedMonitor {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Stops the thread with an asynchronous exception, as if done by
/// `java.lang.Thread.stop`
#[jdwp_command((), 11, 10)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Stop {
    /// The thread object ID.
    pub thread: ThreadID,
    /// Asynchronous exception.
    ///
    /// This object must be an instance of `java.lang.Throwable` or a subclass
    pub throwable: TaggedObjectID,
}

/// Interrupt the thread, as if done by `java.lang.Thread.interrupt`
#[jdwp_command((), 11, 11)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Interrupt {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Get the suspend count for this thread.
///
/// The suspend count is the number of times the thread has been suspended
/// through the thread-level or VM-level suspend commands without a
/// corresponding resume
#[jdwp_command(u32, 11, 12)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct SuspendCount {
    /// The thread object ID.
    pub thread: ThreadID,
}

/// Returns monitor objects owned by the thread, along with stack depth at which
/// the monitor was acquired.
///
/// Stack depth can be unknown (e.g., for monitors acquired by JNI
/// MonitorEnter). The thread must be suspended, and the returned information is
/// relevant only while the thread is suspended.
///
/// Requires `can_get_monitor_frame_info` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
///
/// Since JDWP version 1.6.
#[jdwp_command(Vec<(TaggedObjectID, StackDepth)>, 11, 13)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct OwnedMonitorsStackDepthInfo {
    /// The thread object ID.
    pub thread: ThreadID,
}

#[derive(Debug, Clone)]
pub enum StackDepth {
    Depth(u32),
    Unknown,
}

impl JdwpReadable for StackDepth {
    fn read<R: std::io::Read>(read: &mut JdwpReader<R>) -> std::io::Result<Self> {
        let depth = match i32::read(read)? {
            -1 => StackDepth::Unknown,
            n => StackDepth::Depth(n as u32),
        };
        Ok(depth)
    }
}

/// Force a method to return before it reaches a return statement.
///
/// The method which will return early is referred to as the called method. The
/// called method is the current method (as defined by the Frames section in The
/// Java™ Virtual Machine Specification) for the specified thread at the time
/// this command is received.
///
/// The specified thread must be suspended. The return occurs when execution of
/// Java programming language code is resumed on this thread. Between sending
/// this command and resumption of thread execution, the state of the stack is
/// undefined.
///
/// No further instructions are executed in the called method. Specifically,
/// finally blocks are not executed. Note: this can cause inconsistent states in
/// the application.
///
/// A lock acquired by calling the called method (if it is a synchronized
/// method) and locks acquired by entering synchronized blocks within the called
/// method are released. Note: this does not apply to JNI locks or
/// java.util.concurrent.locks locks.
///
/// Events, such as [MethodExit](super::event::Event::MethodExit), are generated
/// as they would be in a normal return.
///
/// The called method must be a non-native Java programming language method.
/// Forcing return on a thread with only one frame on the stack causes the
/// thread to exit when resumed.
///
/// For void methods, the value must be a void value. For methods that return
/// primitive values, the value's type must match the return type exactly. For
/// object values, there must be a widening reference conversion from the
/// value's type to the return type type and the return type must be loaded.
///
/// Since JDWP version 1.6. Requires `can_force_early_return` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[jdwp_command((), 11, 14)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ForceEarlyReturn {
    /// The thread object ID.
    pub thread: ThreadID,
    /// The value to return.
    pub value: Value,
}
