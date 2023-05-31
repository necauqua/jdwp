use crate::{
    codec::JdwpReadable,
    enums::{ClassStatus, EventKind, SuspendPolicy},
    types::{
        FieldID, Location, ReferenceTypeID, RequestID, TaggedObjectID, TaggedReferenceTypeID,
        ThreadID, Value,
    },
};

use super::jdwp_command;

#[derive(Debug, Clone, JdwpReadable)]
#[repr(u8)]
pub enum Event {
    /// Notification of step completion in the target VM.
    ///
    /// The step event is generated before the code at its location is executed.
    SingleStep(
        /// Request that generated the event
        RequestID,
        /// Stepped thread
        ThreadID,
        /// Location stepped to
        Location,
    ) = EventKind::SingleStep as u8,
    /// Notification of a breakpoint in the target VM.
    ///
    /// The breakpoint event is generated before the code at its location is
    /// executed.
    Breakpoint(
        /// Request that generated the event
        RequestID,
        /// Thread which hit breakpoint
        ThreadID,
        /// Location hit
        Location,
    ) = EventKind::Breakpoint as u8,
    /// Notification of a method invocation in the target VM.
    ///
    /// This event is generated before any code in the invoked method has
    /// executed.
    ///
    /// Method entry events are generated for both native and non-native
    /// methods.
    ///
    /// In some VMs method entry events can occur for a particular thread before
    /// its thread start event occurs if methods are called as part of the
    /// thread's initialization.
    MethodEntry(
        /// Request that generated the event
        RequestID,
        /// Thread which entered method
        ThreadID,
        /// The initial executable location in the method
        Location,
    ) = EventKind::MethodEntry as u8,
    /// Notification of a method return in the target VM.
    ///
    /// This event is generated after all code in the method has executed, but
    /// the location of this event is the last executed location in the
    /// method.
    ///
    /// Method exit events are generated for both native and non-native methods.
    ///
    /// Method exit events are not generated if the method terminates with a
    /// thrown exception.
    MethodExit(
        /// Request that generated the event
        RequestID,
        /// Thread which exited method
        ThreadID,
        /// Location of exit
        Location,
    ) = EventKind::MethodExit as u8,
    /// Notification of a method return in the target VM.
    ///
    /// This event is generated after all code in the method has executed, but
    /// the location of this event is the last executed location in the
    /// method.
    ///
    /// Method exit events are generated for both native and non-native methods.
    ///
    /// Method exit events are not generated if the method terminates with a
    /// thrown exception.
    ///
    /// Since JDWP version 1.6.
    MethodExitWithReturnValue(
        /// Request that generated the event
        RequestID,
        /// Thread which exited method
        ThreadID,
        /// Location of exit
        Location,
        /// Value that will be returned by the method
        Value,
    ) = EventKind::MethodExitWithReturnValue as u8,
    /// Notification that a thread in the target VM is attempting to enter a
    /// monitor that is already acquired by another thread.
    ///
    /// Requires `can_request_monitor_events` capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    ///
    /// Since JDWP version 1.6.
    MonitorContendedEnter(
        /// Request that generated the event
        RequestID,
        /// Thread which is trying to enter the monitor
        ThreadID,
        /// Monitor object reference
        TaggedObjectID,
        /// Location of contended monitor enter
        Location,
    ) = EventKind::MonitorContendedEnter as u8,
    /// Notification of a thread in the target VM is entering a monitor after
    /// waiting for it to be released by another thread.
    ///
    /// Requires `can_request_monitor_events` capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    ///
    /// Since JDWP version 1.6.
    MonitorContendedEntered(
        /// Request that generated the event
        RequestID,
        /// Thread which entered monitor
        ThreadID,
        /// Monitor object reference
        TaggedObjectID,
        /// Location of contended monitor enter
        Location,
    ) = EventKind::MonitorContendedEntered as u8,
    /// Notification of a thread about to wait on a monitor object.
    ///
    /// Requires `can_request_monitor_events` capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    ///
    /// Since JDWP version 1.6.
    MonitorWait(
        /// Request that generated the event
        RequestID,
        /// Thread which is about to wait
        ThreadID,
        /// Monitor object reference
        TaggedObjectID,
        /// Location at which the wait will occur
        Location,
        /// Thread wait time in milliseconds
        u64,
    ) = EventKind::MonitorWait as u8,
    /// Notification that a thread in the target VM has finished waiting on a
    /// monitor object.
    ///
    /// Requires `can_request_monitor_events` capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    ///
    /// Since JDWP version 1.6.
    MonitorWaited(
        /// Request that generated the event
        RequestID,
        /// Thread which waited
        ThreadID,
        /// Monitor object reference
        TaggedObjectID,
        /// Location at which the wait occurred
        Location,
        /// True if timed out
        bool,
    ) = EventKind::MonitorWaited as u8,
    /// Notification of an exception in the target VM.
    ///
    /// If the exception is thrown from a non-native method, the exception event
    /// is generated at the location where the exception is thrown.
    ///
    /// If the exception is thrown from a native method, the exception event is
    /// generated at the first non-native location reached after the exception
    /// is thrown.
    Exception(
        /// Request that generated the event
        RequestID,
        /// Thread with exception
        ThreadID,
        /// Location of exception throw (or first non-native location after
        /// throw if thrown from a native method)
        Location,
        /// Thrown exception
        TaggedObjectID,
        /// Location of catch if caught.
        ///
        /// An exception is considered to be caught if, at the point of the
        /// throw, the current location is dynamically enclosed in a try
        /// statement that handles the exception. (See the JVM
        /// specification for details). If there is such a try
        /// statement, the catch location is the first location in the
        /// appropriate catch clause.
        ///
        /// If there are native methods in the call stack at the time of the
        /// exception, there are important restrictions to note about the
        /// returned catch location.
        ///
        /// In such cases, it is not possible to predict whether an exception
        /// will be handled by some native method on the call stack.
        ///
        /// Thus, it is possible that exceptions considered uncaught here will,
        /// in fact, be handled by a native method and not cause
        /// termination of the target VM.
        ///
        /// Furthermore, it cannot be assumed that the catch location returned
        /// here will ever be reached by the throwing thread. If there
        /// is a native frame between the current location and the catch
        /// location, the exception might be handled and cleared in that
        /// native method instead.
        ///
        /// Note that compilers can generate try-catch blocks in some cases
        /// where they are not explicit in the source code; for example,
        /// the code generated for synchronized and finally blocks can
        /// contain implicit try-catch blocks.
        ///
        /// If such an implicitly generated try-catch is present on the call
        /// stack at the time of the throw, the exception will be
        /// considered caught even though it appears to be uncaught from
        /// examination of the source code.
        Option<Location>,
    ) = EventKind::Exception as u8,
    /// Notification of a new running thread in the target VM.
    ///
    /// The new thread can be the result of a call to `java.lang.Thread.start`
    /// or the result of attaching a new thread to the VM though JNI.
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
    /// [AllThreads](super::virtual_machine::AllThreads) command to return a
    /// thread before its thread start event is received.
    ///
    /// Note that this event gives no information about the creation of the
    /// thread object which may have happened much earlier, depending on the
    /// VM being debugged.
    ThreadStart(
        /// Request that generated the event
        RequestID,
        /// Started thread
        ThreadID,
    ) = EventKind::ThreadStart as u8,
    /// Notification of a completed thread in the target VM.
    ///
    /// The notification is generated by the dying thread before it terminates.
    ///
    /// Because of this timing, it is possible for
    /// [AllThreads](super::virtual_machine::AllThreads) to return this thread
    /// after this event is received.
    ///
    /// Note that this event gives no information about the lifetime of the
    /// thread object.
    ///
    /// It may or may not be collected soon depending on what references exist
    /// in the target VM.
    ThreadDeath(
        /// Request that generated the event
        RequestID,
        /// Ending thread
        ThreadID,
    ) = EventKind::ThreadDeath as u8,
    /// Notification of a class prepare in the target VM.
    ///
    /// See the JVM specification for a definition of class preparation.
    ///
    /// Class prepare events are not generated for primitive classes
    /// (for example, `java.lang.Integer.TYPE`).
    ClassPrepare(
        /// Request that generated the event
        RequestID,
        /// Preparing thread.
        ///
        /// In rare cases, this event may occur in a debugger system thread
        /// within the target VM.
        ///
        /// Debugger threads take precautions to prevent these events, but they
        /// cannot be avoided under some conditions, especially for some
        /// subclasses of `java.lang.Error`.
        ///
        /// If the event was generated by a debugger system thread, the value
        /// returned by this method is null, and if the requested suspend policy
        /// for the event was [EventThread](SuspendPolicy::EventThread)
        /// all threads will be suspended instead, and the composite event's
        /// suspend policy will reflect this change.
        ///
        /// Note that the discussion above does not apply to system threads
        /// created by the target VM during its normal (non-debug)
        /// operation.
        ThreadID,
        /// Type being prepared
        TaggedReferenceTypeID,
        /// Type signature
        String,
        /// Status of type
        ClassStatus,
    ) = EventKind::ClassPrepare as u8,
    /// Notification of a class unload in the target VM.
    ///
    /// There are severe constraints on the debugger back-end during garbage
    /// collection, so unload information is greatly limited.
    ClassUnload(
        /// Request that generated the event
        RequestID,
        /// Type signature
        String,
    ) = EventKind::ClassUnload as u8,
    /// Notification of a field access in the target VM.
    ///
    /// Field modifications are not considered field accesses.
    ///
    /// Requires `can_watch_field_access` capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    FieldAccess(
        /// Request that generated the event
        RequestID,
        /// Accessing thread
        ThreadID,
        /// Location of access
        Location,
        /// Field being accessed
        (ReferenceTypeID, FieldID),
        /// Object being accessed (None for statics)
        Option<TaggedObjectID>,
    ) = EventKind::FieldAccess as u8,
    /// Notification of a field modification in the target VM. Requires
    /// `can_watch_field_modification` capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    FieldModification(
        /// Request that generated the event
        RequestID,
        /// Modifying thread
        ThreadID,
        /// Location of modify
        Location,
        /// Field being modified
        (TaggedReferenceTypeID, FieldID),
        /// Object being modified (None for statics)
        Option<TaggedObjectID>,
        /// Value to be assigned
        Value,
    ) = EventKind::FieldModification as u8,
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
    VmStart(
        /// Request that generated the event (or None if this event is
        /// automatically generated)
        Option<RequestID>,
        /// Initial thread
        ThreadID,
    ) = EventKind::VmStart as u8,
    VmDeath(
        /// Request that generated the event
        RequestID,
    ) = EventKind::VmDeath as u8,
}

#[jdwp_command((), 64, 100)]
#[derive(Debug, Clone, JdwpReadable)]
pub struct Composite {
    pub suspend_policy: SuspendPolicy,
    pub events: Vec<Event>,
}
