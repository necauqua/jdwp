use super::jdwp_command;
use crate::{
    codec::{JdwpReadable, JdwpWritable},
    enums::InvokeOptions,
    types::{
        ClassID, FieldID, InvokeMethodReply, ObjectID, TaggedObjectID, TaggedReferenceTypeID,
        ThreadID, UntaggedValue, Value,
    },
};

#[jdwp_command(TaggedReferenceTypeID, 9, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ReferenceType {
    /// The object ID
    object: ObjectID,
}

/// Returns the value of one or more instance fields.
///
/// Each field must be member of the object's type or one of its superclasses,
/// superinterfaces, or implemented interfaces. Access control is not enforced;
/// for example, the values of private fields can be obtained.
#[jdwp_command(Vec<Value>, 9, 2)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct GetValues {
    /// The object ID
    object: ObjectID,
    /// Fields to get
    fields: Vec<FieldID>,
}

/// Sets the value of one or more instance fields.
///
/// Each field must be member of the object's type or one of its superclasses,
/// superinterfaces, or implemented interfaces. Access control is not enforced;
/// for example, the values of private fields can be set. For primitive values,
/// the value's type must match the field's type exactly. For object values,
/// there must be a widening reference conversion from the value's type to the
/// field's type and the field's type must be loaded.
#[jdwp_command((), 9, 3)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct SetValues {
    /// The object ID
    object: ObjectID,
    /// Fields and the values to set them to
    fields: Vec<(FieldID, UntaggedValue)>,
}

/// Returns monitor information for an object.
///
/// All threads in the VM must be suspended.
///
/// Requires `can_get_monitor_info` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[jdwp_command(9, 5)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct MonitorInfo {
    /// The object ID
    object: ObjectID,
}

#[derive(Debug, JdwpReadable)]
pub struct MonitorInfoReply {
    /// The monitor owner, or null if it is not currently owned
    pub owner: Option<ThreadID>,
    /// The number of times the monitor has been entered.
    pub entry_count: i32,
    /// The threads that are waiting for the monitor 0 if there is no current
    /// owner
    pub waiters: Vec<ThreadID>,
}

/// Invokes a instance method.
///
/// The method must be member of the object's type or one of its superclasses,
/// superinterfaces, or implemented interfaces. Access control is not enforced;
/// for example, private methods can be invoked.
///
/// The method invocation will occur in the specified thread. Method invocation
/// can occur only if the specified thread has been suspended by an event.
/// Method invocation is not supported when the target VM has been suspended by
/// the front-end.
///
/// The specified method is invoked with the arguments in the specified argument
/// list. The method invocation is synchronous; the reply packet is not sent
/// until the invoked method returns in the target VM. The return value
/// (possibly the void value) is included in the reply packet.
///
/// For primitive arguments, the argument value's type must match the argument's
/// type exactly. For object arguments, there must be a widening reference
/// conversion from the argument value's type to the argument's type and the
/// argument's type must be loaded.
///
/// By default, all threads in the target VM are resumed while the method is
/// being invoked if they were previously suspended by an event or by a command.
/// This is done to prevent the deadlocks that will occur if any of the threads
/// own monitors that will be needed by the invoked method. It is possible that
/// breakpoints or other events might occur during the invocation. Note,
/// however, that this implicit resume acts exactly like the ThreadReference
/// resume command, so if the thread's suspend count is greater than 1, it will
/// remain in a suspended state during the invocation. By default, when the
/// invocation completes, all threads in the target VM are suspended, regardless
/// their state before the invocation.
///
/// The resumption of other threads during the invoke can be prevented by
/// specifying the INVOKE_SINGLE_THREADED bit flag in the options field;
/// however, there is no protection against or recovery from the deadlocks
/// described above, so this option should be used with great caution. Only the
/// specified thread will be resumed (as described for all threads above). Upon
/// completion of a single threaded invoke, the invoking thread will be
/// suspended once again. Note that any threads started during the single
/// threaded invocation will not be suspended when the invocation completes.
///
/// If the target VM is disconnected during the invoke (for example, through the
/// VirtualMachine dispose command) the method invocation continues.
#[jdwp_command(9, 6)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct InvokeMethod {
    /// The object ID
    object: ObjectID,
    /// The thread in which to invoke
    thread: ThreadID,
    /// The class type
    class: ClassID,
    /// The method to invoke
    method: FieldID,
    /// The arguments
    arguments: Vec<Value>,
    /// Invocation options
    options: InvokeOptions,
}

/// Prevents garbage collection for the given object.
///
/// By default all objects in back-end replies may be collected at any time the
/// target VM is running. A call to this command guarantees that the object will
/// not be collected. The [EnableCollection] command can be used to allow
/// collection once again.
///
/// Note that while the target VM is suspended, no garbage collection will occur
/// because all threads are suspended. The typical examination of variables,
/// fields, and arrays during the suspension is safe without explicitly
/// disabling garbage collection.
///
/// This method should be used sparingly, as it alters the pattern of garbage
/// collection in the target VM and, consequently, may result in application
/// behavior under the debugger that differs from its non-debugged behavior.
#[jdwp_command((), 9, 7)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct DisableCollection {
    /// The object ID
    object: ObjectID,
}

/// Permits garbage collection for this object.
///
/// By default all objects returned by JDWP may become unreachable in the target
/// VM, and hence may be garbage collected. A call to this command is necessary
/// only if garbage collection was previously disabled with the
/// [DisableCollection] command.
#[jdwp_command((), 9, 8)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct EnableCollection {
    /// The object ID
    object: ObjectID,
}

/// Determines whether an object has been garbage collected in the target VM.
#[jdwp_command(bool, 9, 9)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct IsCollected {
    /// The object ID
    object: ObjectID,
}

/// Returns objects that directly reference this object. Only objects that are
/// reachable for the purposes of garbage collection are returned. Note that an
/// object can also be referenced in other ways, such as from a local variable
/// in a stack frame, or from a JNI global reference. Such non-object referrers
/// are not returned by this command.
///
/// Since JDWP version 1.6.
///
/// Requires `can_get_instance_info` capability - see
/// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[jdwp_command(Vec<TaggedObjectID>, 9, 10)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ReferringObjects {
    /// The object ID
    object: ObjectID,
    /// Maximum number of referring objects to return. Must be non-negative. If
    /// zero, all referring objects are returned.
    max_referrers: u32,
}
