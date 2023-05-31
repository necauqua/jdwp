use std::{fmt::Debug, marker::PhantomData};

use crate::{
    codec::{JdwpReadable, JdwpWritable},
    enums::ClassStatus,
    functional::{Coll, Single},
    types::{ObjectID, ReferenceTypeID, StringID, TaggedReferenceTypeID, ThreadGroupID, ThreadID},
};

use super::jdwp_command;

/// Returns the JDWP version implemented by the target VM.
///
/// The version string format is implementation dependent.
#[jdwp_command(1, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Version;

#[derive(Debug, JdwpReadable)]
pub struct VersionReply {
    /// Text information on the VM version
    pub description: String,
    /// Major JDWP Version number
    pub version_major: u32,
    /// Minor JDWP Version number
    pub version_minor: u32,
    /// Target VM JRE version, as in the java.version property
    pub vm_version: String,
    /// Target VM name, as in the java.vm.name property
    pub vm_name: String,
}

/// Returns reference types for all the classes loaded by the target VM which
/// match the given signature.
///
/// Multiple reference types will be returned if two or more class loaders have
/// loaded a class of the same name.
///
/// The search is confined to loaded classes only; no attempt is made to load a
/// class of the given signature.
#[jdwp_command(C, 1, 2)]
#[derive(Clone, JdwpWritable)]
pub struct ClassesBySignatureGeneric<C: Coll<Item = (TaggedReferenceTypeID, ClassStatus)>> {
    /// JNI signature of the class to find (for example, "Ljava/lang/String;")
    signature: String,
    _phantom: PhantomData<C>,
}

/// This is needed because inference cannot guess what you need since there are
/// no parameters
/// And the Single helper type is in a private jdwp module
pub type ClassBySignature = ClassesBySignatureGeneric<Single<(TaggedReferenceTypeID, ClassStatus)>>;

impl Debug for ClassBySignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassBySignature")
            .field("signature", &self.signature)
            .finish()
    }
}

/// The inference is able to figure out N by the destructuring pattern
pub type ClassesBySignatureStatic<const N: usize> =
    ClassesBySignatureGeneric<[(TaggedReferenceTypeID, ClassStatus); N]>;

impl<const N: usize> Debug for ClassesBySignatureStatic<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("ClassesBySignatureStatic<{}>", N)) // hope this const-optimizes?.
            .field("signature", &self.signature)
            .finish()
    }
}

/// The 'standard' variant with a vector
pub type ClassesBySignature = ClassesBySignatureGeneric<Vec<(TaggedReferenceTypeID, ClassStatus)>>;

impl Debug for ClassesBySignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassesBySignature")
            .field("signature", &self.signature)
            .finish()
    }
}

/// Returns reference types for all classes currently loaded by the target VM.
#[jdwp_command(Vec<Class>, 1, 3)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct AllClasses;

#[derive(Debug, JdwpReadable)]
pub struct Class {
    /// Matching loaded reference type
    pub type_id: TaggedReferenceTypeID,
    /// The JNI signature of the loaded reference type
    pub signature: String,
    /// The current class status
    pub status: ClassStatus,
}

/// Returns all threads currently running in the target VM.
///
/// The returned list contains threads created through java.lang.Thread, all
/// native threads attached to the target VM through JNI, and system threads
/// created by the target VM.
///
/// Threads that have not yet been started and threads that have completed
/// their execution are not included in the returned list.
#[jdwp_command(Vec<ThreadID>, 1, 4)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct AllThreads;

/// Returns all thread groups that do not have a parent. This command may be
/// used as the first step in building a tree (or trees) of the existing thread
/// groups.
#[jdwp_command(Vec<ThreadGroupID>, 1, 5)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct TopLevelThreadGroups;

/// Invalidates this virtual machine mirror.
///
/// The communication channel to the target VM is closed, and the target VM
/// prepares to accept another subsequent connection from this debugger or
/// another debugger, including the following tasks:
/// - All event requests are cancelled.
/// - All threads suspended by the thread-level resume command or the VM-level
/// resume command are resumed as many times as necessary for them to run.
/// - Garbage collection is re-enabled in all cases where it was disabled
///
/// Any current method invocations executing in the target VM are continued
/// after the disconnection. Upon completion of any such method invocation,
/// the invoking thread continues from the location where it was originally
/// stopped.
///
/// Resources originating in this VirtualMachine (ObjectReferences,
/// ReferenceTypes, etc.) will become invalid.
#[jdwp_command((), 1, 6)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Dispose;

/// Returns the sizes of variably-sized data types in the target VM.
///
/// The returned values indicate the number of bytes used by the identifiers in
/// command and reply packets.
#[jdwp_command(IDSizeInfo, 1, 7)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct IDSizes;

#[derive(Debug, Clone, JdwpReadable)]
pub struct IDSizeInfo {
    /// field_id size in bytes
    pub field_id_size: u32,
    /// method_id size in bytes
    pub method_id_size: u32,
    /// object_id size in bytes
    pub object_id_size: u32,
    /// reference_type_id size in bytes
    pub reference_type_id_size: u32,
    /// frame_id size in bytes
    pub frame_id_size: u32,
}

/// Suspends the execution of the application running in the target VM.
/// All Java threads currently running will be suspended.
///
/// Unlike java.lang.Thread.suspend, suspends of both the virtual machine and
/// individual threads are counted. Before a thread will run again, it must
/// be resumed through the VM-level resume command or the thread-level resume
/// command the same number of times it has been suspended.
#[jdwp_command((), 1, 8)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Suspend;

/// Resumes execution of the application after the suspend command or an event
/// has stopped it.
///
/// Suspensions of the Virtual Machine and individual threads are counted.
///
/// If a particular thread is suspended n times, it must resumed n times before
/// it will continue.
#[jdwp_command((), 1, 9)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Resume;

/// Terminates the target VM with the given exit code.
///
/// On some platforms, the exit code might be truncated, for example, to the
/// low order 8 bits.
///
/// All ids previously returned from the target VM become invalid.
///
/// Threads running in the VM are abruptly terminated.
///
/// A thread death exception is not thrown and finally blocks are not run.
#[jdwp_command((), 1, 10)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Exit {
    exit_code: i32,
}

/// Creates a new string object in the target VM and returns its id.
#[jdwp_command(StringID, 1, 11)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct CreateString {
    /// UTF-8 characters to use in the created string
    string: String,
}

/// Retrieve this VM's capabilities.
///
/// The capabilities are returned as booleans, each indicating the presence or
/// absence of a capability.
///
/// The commands associated with each capability will return the
/// NOT_IMPLEMENTED error if the capability is not available.
#[jdwp_command(1, 12)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Capabilities;

#[derive(Debug, JdwpReadable)]
pub struct CapabilitiesReply {
    /// Can the VM watch field modification, and therefore can it send the
    /// Modification Watchpoint Event?
    pub can_watch_field_modification: bool,
    /// Can the VM watch field access, and therefore can it send the
    /// Access Watchpoint Event?
    pub can_watch_field_access: bool,
    /// Can the VM get the bytecodes of a given method?
    pub can_get_bytecodes: bool,
    /// Can the VM determine whether a field or method is synthetic?
    /// (that is, can the VM determine if the method or the field was invented
    /// by the compiler?)
    pub can_get_synthetic_attribute: bool,
    /// Can the VM get the owned monitors information for a thread?
    pub can_get_owned_monitor_info: bool,
    /// Can the VM get the current contended monitor of a thread?
    pub can_get_current_contended_monitor: bool,
    /// Can the VM get the monitor information for a given object?
    pub can_get_monitor_info: bool,
}

/// Retrieve the classpath and bootclasspath of the target VM.
///
/// If the classpath is not defined, returns an empty list.
///
/// If the bootclasspath is not defined returns an empty list.
#[jdwp_command(1, 13)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ClassPaths;

#[derive(Debug, JdwpReadable)]
pub struct ClassPathsReply {
    /// Base directory used to resolve relative paths in either of the following
    /// lists.
    pub base_dir: String,
    /// Components of the classpath
    pub classpaths: Vec<String>,
    /// Components of the bootclasspath
    pub bootclasspaths: Vec<String>,
}

/// Releases a list of object IDs.
///
/// For each object in the list, the following applies.
///
/// The count of references held by the back-end (the reference count) will be
/// decremented by ref_cnt.
///
/// If thereafter the reference count is less than or equal to zero, the ID is
/// freed.
///
/// Any back-end resources associated with the freed ID may be freed, and if
/// garbage collection was disabled for the object, it will be re-enabled.
///
/// The sender of this command promises that no further commands will be sent
/// referencing a freed ID.
///
/// Use of this command is not required.
///
/// If it is not sent, resources associated with each ID will be freed by the
/// back-end at some time after the corresponding object is garbage collected.
///
/// It is most useful to use this command to reduce the load on the back-end if
/// a very large number of objects has been retrieved from the back-end (a large
/// array, for example) but may not be garbage collected any time soon.
///
/// IDs may be re-used by the back-end after they have been freed with this
/// command.
///
/// This description assumes reference counting, a back-end may use any
/// implementation which operates equivalently.
#[jdwp_command((), 1, 14)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct DisposeObjects<'a> {
    requests: &'a [(ObjectID, u32)],
}

/// Tells the target VM to stop sending events. Events are not discarded; they
/// are held until a subsequent ReleaseEvents command is sent.
///
/// This command is useful to control the number of events sent to the debugger
/// VM in situations where very large numbers of events are generated.
///
/// While events are held by the debugger back-end, application execution may
/// be frozen by the debugger back-end to prevent buffer overflows on the back
/// end.
///
/// Responses to commands are never held and are not affected by this command.
/// If events are already being held, this command is ignored.
#[jdwp_command((), 1, 15)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct HoldEvents;

/// Tells the target VM to continue sending events.
///
/// This command is used to restore normal activity after a HoldEvents command.
///
/// If there is no current HoldEvents command in effect, this command is
/// ignored.
#[jdwp_command((), 1, 16)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ReleaseEvents;

/// Retrieve all of this VM's capabilities.
///
/// The capabilities are returned as booleans, each indicating the presence or
/// absence of a capability.
///
/// The commands associated with each capability will return the
/// NOT_IMPLEMENTED error if the capability is not available.
///
/// Since JDWP version 1.4.
#[jdwp_command(1, 17)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct CapabilitiesNew;

#[derive(JdwpReadable)]
pub struct CapabilitiesNewReply {
    /// The prefix of [CapabilitiesNew] is identical to that of old
    /// [Capabilities]
    pub capabilities: CapabilitiesReply,
    /// Can the VM redefine classes?
    pub can_redefine_classes: bool,
    /// Can the VM add methods when redefining classes?
    pub can_add_method: bool,
    /// Can the VM redefine classes in arbitrary ways?
    pub can_unrestrictedly_redefine_classes: bool,
    /// Can the VM pop stack frames?
    pub can_pop_frames: bool,
    /// Can the VM filter events by specific object?
    pub can_use_instance_filters: bool,
    /// Can the VM get the source debug extension?
    pub can_get_source_debug_extension: bool,
    /// Can the VM request VM death events?
    pub can_request_vmdeath_event: bool,
    /// Can the VM set a default stratum?
    pub can_set_default_stratum: bool,
    /// Can the VM return instances, counts of instances of classes and
    /// referring objects?
    pub can_get_instance_info: bool,
    /// Can the VM request monitor events?
    pub can_request_monitor_events: bool,
    /// Can the VM get monitors with frame depth info?
    pub can_get_monitor_frame_info: bool,
    /// Can the VM filter class prepare events by source name?
    pub can_use_source_name_filters: bool,
    /// Can the VM return the constant pool information?
    pub can_get_constant_pool: bool,
    /// Can the VM force early return from a method?
    pub can_force_early_return: bool,
    /// Reserved for future capability
    _reserved_22: bool,
    /// Reserved for future capability
    _reserved_23: bool,
    /// Reserved for future capability
    _reserved_24: bool,
    /// Reserved for future capability
    _reserved_25: bool,
    /// Reserved for future capability
    _reserved_26: bool,
    /// Reserved for future capability
    _reserved_27: bool,
    /// Reserved for future capability
    _reserved_28: bool,
    /// Reserved for future capability
    _reserved_29: bool,
    /// Reserved for future capability
    _reserved_30: bool,
    /// Reserved for future capability
    _reserved_31: bool,
    /// Reserved for future capability
    _reserved_32: bool,
}

// skip reserved fields from Debug
impl Debug for CapabilitiesNewReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilitiesNewReply")
            .field("capabilities", &self.capabilities)
            .field("can_redefine_classes", &self.can_redefine_classes)
            .field("can_add_method", &self.can_add_method)
            .field(
                "can_unrestrictedly_redefine_classes",
                &self.can_unrestrictedly_redefine_classes,
            )
            .field("can_pop_frames", &self.can_pop_frames)
            .field("can_use_instance_filters", &self.can_use_instance_filters)
            .field(
                "can_get_source_debug_extension",
                &self.can_get_source_debug_extension,
            )
            .field("can_request_vmdeath_event", &self.can_request_vmdeath_event)
            .field("can_set_default_stratum", &self.can_set_default_stratum)
            .field("can_get_instance_info", &self.can_get_instance_info)
            .field(
                "can_request_monitor_events",
                &self.can_request_monitor_events,
            )
            .field(
                "can_get_monitor_frame_info",
                &self.can_get_monitor_frame_info,
            )
            .field(
                "can_use_source_name_filters",
                &self.can_use_source_name_filters,
            )
            .field("can_get_constant_pool", &self.can_get_constant_pool)
            .field("can_force_early_return", &self.can_force_early_return)
            .finish()
    }
}

/// Installs new class definitions.
///
/// If there are active stack frames in methods of the redefined classes in the
/// target VM then those active frames continue to run the bytecodes of the
/// original method. These methods are considered obsolete - see
/// [IsObsolete](super::method::IsObsolete).
///
/// The methods in the redefined classes will be used for new invokes in the
/// target VM. The original method ID refers to the redefined method.
///
/// All breakpoints in the redefined classes are cleared.
///
/// If resetting of stack frames is desired, the PopFrames command can be
/// used to pop frames with obsolete methods.
///
/// Requires `can_redefine_classes` capability - see [CapabilitiesNew].
///
/// In addition to the `can_redefine_classes` capability, the target VM must
/// have the `can_add_method` capability to add methods when redefining classes,
/// or the `can_unrestrictedly_redefine_classes` to redefine classes in
/// arbitrary ways.
#[jdwp_command((), 1, 18)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct RedefineClasses<'a> {
    classes: &'a [(ReferenceTypeID, Vec<u8>)],
}

/// Set the default stratum. Requires `can_set_default_stratum` capability -
/// see [CapabilitiesNew].
#[jdwp_command((), 1, 19)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct SetDefaultStratum {
    /// default stratum, or empty string to use reference type default.
    stratum_id: String,
}

/// Returns reference types for all classes currently loaded by the target VM.
///
/// Both the JNI signature and the generic signature are returned for each
/// class.
///
/// Generic signatures are described in the signature attribute section in
/// The Java™ Virtual Machine Specification.
///
/// Since JDWP version 1.5.
#[jdwp_command(Vec<GenericClass>, 1, 20)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct AllClassesWithGeneric;

#[derive(Debug, JdwpReadable)]
pub struct GenericClass {
    /// Loaded reference type
    pub type_id: TaggedReferenceTypeID,
    /// The JNI signature of the loaded reference type
    pub signature: String,
    /// The generic signature of the loaded reference type or an empty string if
    /// there is none.
    pub generic_signature: String,
    /// The current class status
    pub status: ClassStatus,
}

/// Returns the number of instances of each reference type in the input list.
///
/// Only instances that are reachable for the purposes of garbage collection
/// are counted.
///
/// If a reference type is invalid, eg. it has been unloaded, zero is returned
/// for its instance count.
///
/// Since JDWP version 1.6. Requires canGetInstanceInfo capability - see
/// [CapabilitiesNew].
#[jdwp_command(C::Map<u64>, 1, 21)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct InstanceCounts<C: Coll<Item = ReferenceTypeID>> {
    /// A list of reference type IDs.
    ref_types: C,
}
