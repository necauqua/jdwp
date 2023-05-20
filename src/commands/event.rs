use std::io::{self, Read, Write};

use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    enums::{ClassStatus, EventKind, SuspendPolicy},
    types::{
        FieldID, Location, ReferenceTypeID, RequestID, TaggedObjectID, TaggedReferenceTypeID,
        ThreadID, Value,
    },
};

use super::jdwp_command;

/// Notification of initialization of a target VM.
///
/// This event is received before the main thread is started and before any
/// application code has been executed.
///
/// Before this event occurs a significant amount of system code has
/// executed and a number of system classes have been loaded.
///
/// This event is always generated by the target VM, even if not explicitly
/// requested.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct VmStart {
    /// Request that generated event (or 0 if this event is automatically generated)
    pub request_id: i32,
    /// Initial thread
    pub thread_id: ThreadID,
}

/// Notification of step completion in the target VM.
///
/// The step event is generated before the code at its location is executed.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct SingleStep {
    /// Request that generated event
    pub request_id: i32,
    /// Stepped thread
    pub thread: ThreadID,
    /// Location stepped to
    pub location: Location,
}

/// Notification of a breakpoint in the target VM.
///
/// The breakpoint event is generated before the code at its location is
/// executed.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct Breakpoint {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which hit breakpoint
    pub thread: ThreadID,
    /// Location hit
    pub location: Location,
}

/// Notification of a method invocation in the target VM.
///
/// This event is generated before any code in the invoked method has executed.
///
/// Method entry events are generated for both native and non-native methods.
///
/// In some VMs method entry events can occur for a particular thread before
/// its thread start event occurs if methods are called as part of the thread's
/// initialization.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct MethodEntry {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which entered method
    pub thread: ThreadID,
    /// The initial executable location in the method
    pub location: Location,
}

/// Notification of a method return in the target VM.
///
/// This event is generated after all code in the method has executed, but the
/// location of this event is the last executed location in the method.
///
/// Method exit events are generated for both native and non-native methods.
///
/// Method exit events are not generated if the method terminates with a thrown
/// exception.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct MethodExit {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which exited method
    pub thread: ThreadID,
    /// Location of exit
    pub location: Location,
}

/// Notification of a method return in the target VM.
///
/// This event is generated after all code in the method has executed, but the
/// location of this event is the last executed location in the method.
///
/// Method exit events are generated for both native and non-native methods.
///
/// Method exit events are not generated if the method terminates with a thrown
/// exception.
///
/// Since JDWP version 1.6.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct MethodExitWithReturnValue {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which exited method
    pub thread: ThreadID,
    /// Location of exit
    pub location: Location,
    /// Value that will be returned by the method
    pub value: Value,
}

/// Notification that a thread in the target VM is attempting to enter a
/// monitor that is already acquired by another thread.
///
/// Requires `can_request_monitor_events` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
///
/// Since JDWP version 1.6.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct MonitorContendedEnter {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which is trying to enter the monitor
    pub thread: ThreadID,
    /// Monitor object reference
    pub object: TaggedObjectID,
    /// Location of contended monitor enter
    pub location: Location,
}

/// Notification of a thread in the target VM is entering a monitor after
/// waiting for it to be released by another thread.
///
/// Requires `can_request_monitor_events` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
///
/// Since JDWP version 1.6.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct MonitorContendedEntered {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which entered monitor
    pub thread: ThreadID,
    /// Monitor object reference
    pub object: TaggedObjectID,
    /// Location of contended monitor enter
    pub location: Location,
}

/// Notification of a thread about to wait on a monitor object.
///
/// Requires `can_request_monitor_events` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
///
/// Since JDWP version 1.6.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct MonitorWait {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which is about to wait
    pub thread: ThreadID,
    /// Monitor object reference
    pub object: TaggedObjectID,
    /// Location at which the wait will occur
    pub location: Location,
    /// Thread wait time in milliseconds
    pub timeout: i64,
}

/// Notification that a thread in the target VM has finished waiting on a
/// monitor object.
///
/// Requires `can_request_monitor_events` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
///
/// Since JDWP version 1.6.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct MonitorWaited {
    /// Request that generated event
    pub request_id: i32,
    /// Thread which waited
    pub thread: ThreadID,
    /// Monitor object reference
    pub object: TaggedObjectID,
    /// Location at which the wait occurred
    pub location: Location,
    /// True if timed out
    pub timed_out: bool,
}

/// Notification of an exception in the target VM.
///
/// If the exception is thrown from a non-native method, the exception event is
/// generated at the location where the exception is thrown.
///
/// If the exception is thrown from a native method, the exception event is
/// generated at the first non-native location reached after the exception is
/// thrown.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct Exception {
    /// Request that generated event
    pub request_id: i32,
    /// Thread with exception
    pub thread: ThreadID,
    /// Location of exception throw (or first non-native location after throw if thrown from a native method)
    pub location: Location,
    /// Thrown exception
    pub exception: TaggedObjectID,
    /// Location of catch if caught.
    ///
    /// An exception is considered to be caught if, at the point of the throw,
    /// the current location is dynamically enclosed in a try statement that
    /// handles the exception. (See the JVM specification for details).
    /// If there is such a try statement, the catch location is the first
    /// location in the appropriate catch clause.
    ///
    /// If there are native methods in the call stack at the time of the
    /// exception, there are important restrictions to note about the returned
    /// catch location.
    ///
    /// In such cases, it is not possible to predict whether an exception will
    /// be handled by some native method on the call stack.
    ///
    /// Thus, it is possible that exceptions considered uncaught here will, in
    /// fact, be handled by a native method and not cause termination of the
    /// target VM.
    ///
    /// Furthermore, it cannot be assumed that the catch location returned here
    /// will ever be reached by the throwing thread. If there is a native frame
    /// between the current location and the catch location, the exception
    /// might be handled and cleared in that native method instead.
    ///
    /// Note that compilers can generate try-catch blocks in some cases where
    /// they are not explicit in the source code; for example, the code
    /// generated for synchronized and finally blocks can contain implicit
    /// try-catch blocks.
    ///
    /// If such an implicitly generated try-catch is present on the call stack
    /// at the time of the throw, the exception will be considered caught even
    /// though it appears to be uncaught from examination of the source code.
    pub catch_location: Option<Location>,
}

/// Notification of a new running thread in the target VM.
///
/// The new thread can be the result of a call to `java.lang.Thread.start` or
/// the result of attaching a new thread to the VM though JNI.
///
/// The notification is generated by the new thread some time before its
/// execution starts.
///
/// Because of this timing, it is possible to receive other events for the
/// thread before this event is received.
///
/// (Notably, Method Entry Events and Method Exit Events might occur during
/// thread initialization.
///
/// It is also possible for the
/// [AllThreads](super::virtual_machine::AllThreads) command to return a thread
/// before its thread start event is received.
///
/// Note that this event gives no information about the creation of the thread
/// object which may have happened much earlier, depending on the VM being
/// debugged.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct ThreadStart {
    /// Request that generated event
    pub request_id: i32,
    /// Started thread
    pub thread: ThreadID,
}

/// Notification of a completed thread in the target VM.
///
/// The notification is generated by the dying thread before it terminates.
///
/// Because of this timing, it is possible for
/// [AllThreads](super::virtual_machine::AllThreads) to return this thread
/// after this event is received.
///
/// Note that this event gives no information about the lifetime of the thread
/// object.
///
/// It may or may not be collected soon depending on what references exist in
/// the target VM.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct ThreadDeath {
    /// Request that generated event
    pub request_id: i32,
    /// Ending thread
    pub thread: ThreadID,
}

/// Notification of a class prepare in the target VM.
///
/// See the JVM specification for a definition of class preparation.
/// 
/// Class prepare events are not generated for primitive classes
/// (for example, `java.lang.Integer.TYPE`).
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct ClassPrepare {
    /// Request that generated event
    pub request_id: i32,
    /// Preparing thread.
    ///
    /// In rare cases, this event may occur in a debugger system thread within
    /// the target VM.
    ///
    /// Debugger threads take precautions to prevent these events, but they
    /// cannot be avoided under some conditions, especially for some subclasses
    /// of `java.lang.Error`.
    ///
    /// If the event was generated by a debugger system thread, the value
    /// returned by this method is null, and if the requested suspend policy
    /// for the event was [EventThread](SuspendPolicy::EventThread)
    /// all threads will be suspended instead, and the composite event's
    /// suspend policy will reflect this change.
    ///
    /// Note that the discussion above does not apply to system threads created
    /// by the target VM during its normal (non-debug) operation.
    pub thread: ThreadID,
    /// Type being prepared
    pub ref_type_id: TaggedReferenceTypeID,
    /// Type signature
    pub signature: String,
    /// Status of type
    pub status: ClassStatus,
}

/// Notification of a class unload in the target VM.
///
/// There are severe constraints on the debugger back-end during garbage
/// collection, so unload information is greatly limited.
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct ClassUnload {
    /// Request that generated event
    pub request_id: i32,
    /// Type signature
    pub signature: String,
}

/// Notification of a field access in the target VM.
///
/// Field modifications are not considered field accesses.
///
/// Requires `can_watch_field_access` capability - see [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct FieldAccess {
    /// Request that generated event
    pub request_id: i32,
    /// Accessing thread
    pub thread: ThreadID,
    /// Location of access
    pub location: Location,
    /// Type of field
    pub ref_type_id: ReferenceTypeID,
    /// Field being accessed
    pub field_id: FieldID,
    /// Object being accessed (None for statics)
    pub object: Option<TaggedObjectID>,
}

#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct FieldModification {
    /// Request that generated event
    pub request_id: RequestID,
    /// Modifying thread
    pub thread: ThreadID,
    /// Location of modify
    pub location: Location,
    /// Type of field
    pub ref_type_id: TaggedReferenceTypeID,
    /// Field being modified
    pub field_id: FieldID,
    /// Object being modified (None for statics)
    pub object: Option<TaggedObjectID>,
    /// Value to be assigned
    pub value: Value,
}

#[derive(Debug, JdwpReadable, JdwpWritable)]
pub struct VmDeath {
    /// Request that generated event
    pub request_id: i32,
}

macro_rules! event_io {
    ($($events:ident),* $(,)?) => {

        #[derive(Debug)]
        pub enum Event {
            $($events($events),)*
        }

        impl JdwpReadable for Event {
            fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
                match EventKind::read(read)? {
                    $(EventKind::$events => Ok(Event::$events($events::read(read)?)),)*
                    _ => Err(io::Error::from(io::ErrorKind::InvalidData))
                }
            }
        }

        impl JdwpWritable for Event {
            fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
                match self {
                    $(
                        Event::$events(e) => {
                            EventKind::$events.write(write)?;
                            e.write(write)?;
                        }
                    )*
                }
                Ok(())
            }
        }
    };
}

event_io! {
    VmStart,
    SingleStep,
    Breakpoint,
    MethodEntry,
    MethodExit,
    MethodExitWithReturnValue,
    MonitorContendedEnter,
    MonitorContendedEntered,
    MonitorWait,
    MonitorWaited,
    Exception,
    ThreadStart,
    ThreadDeath,
    ClassPrepare,
    ClassUnload,
    FieldAccess,
    FieldModification,
    VmDeath,
}

#[jdwp_command((), 64, 100)]
#[derive(Debug, JdwpWritable, JdwpReadable)]
pub struct Composite {
    pub suspend_policy: SuspendPolicy,
    pub events: Vec<Event>,
}
