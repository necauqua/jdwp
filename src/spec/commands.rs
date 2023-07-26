use std::{
    fmt,
    fmt::Debug,
    io::{self, Read},
    marker::PhantomData,
    ops::Deref,
};

use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    functional::{Coll, Single},
    jdwp_command,
    jvm::{FieldModifiers, MethodModifiers, TypeModifiers},
    spec::*,
};

/// VirtualMachine Command Set (1)
pub mod virtual_machine {
    use super::*;

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

    /// Returns reference types for all the classes loaded by the target VM
    /// which match the given signature.
    ///
    /// Multiple reference types will be returned if two or more class loaders
    /// have loaded a class of the same name.
    ///
    /// The search is confined to loaded classes only; no attempt is made to
    /// load a class of the given signature.
    #[jdwp_command(C, 1, 2)]
    #[derive(Clone, JdwpWritable)]
    pub struct ClassesBySignatureGeneric<'a, C: Coll<Item = (TaggedReferenceTypeID, ClassStatus)>> {
        /// JNI signature of the class to find (for example,
        /// "Ljava/lang/String;")
        signature: &'a str,
        _phantom: PhantomData<C>,
    }

    /// This is needed because inference cannot guess what you need since there
    /// are no parameters
    /// And the Single helper type is in a private jdwp module
    pub type ClassBySignature<'a> =
        ClassesBySignatureGeneric<'a, Single<(TaggedReferenceTypeID, ClassStatus)>>;

    impl<'a> Debug for ClassBySignature<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ClassBySignature")
                .field("signature", &self.signature)
                .finish()
        }
    }

    /// The inference is able to figure out N by the destructuring pattern
    pub type ClassesBySignatureStatic<'a, const N: usize> =
        ClassesBySignatureGeneric<'a, [(TaggedReferenceTypeID, ClassStatus); N]>;

    impl<'a, const N: usize> Debug for ClassesBySignatureStatic<'a, N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct(&format!("ClassesBySignatureStatic<{}>", N)) // hope this const-optimizes?.
                .field("signature", &self.signature)
                .finish()
        }
    }

    /// The 'standard' variant with a vector
    pub type ClassesBySignature<'a> =
        ClassesBySignatureGeneric<'a, Vec<(TaggedReferenceTypeID, ClassStatus)>>;

    impl<'a> Debug for ClassesBySignature<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ClassesBySignature")
                .field("signature", &self.signature)
                .finish()
        }
    }

    /// Returns reference types for all classes currently loaded by the target
    /// VM.
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
    /// used as the first step in building a tree (or trees) of the existing
    /// thread groups.
    #[jdwp_command(Vec<ThreadGroupID>, 1, 5)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct TopLevelThreadGroups;

    /// Invalidates this virtual machine mirror.
    ///
    /// The communication channel to the target VM is closed, and the target VM
    /// prepares to accept another subsequent connection from this debugger or
    /// another debugger, including the following tasks:
    /// - All event requests are cancelled.
    /// - All threads suspended by the thread-level resume command or the
    ///   VM-level
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
    /// The returned values indicate the number of bytes used by the identifiers
    /// in command and reply packets.
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

    impl Default for IDSizeInfo {
        fn default() -> Self {
            Self {
                field_id_size: 8,
                method_id_size: 8,
                object_id_size: 8,
                reference_type_id_size: 8,
                frame_id_size: 8,
            }
        }
    }

    /// Suspends the execution of the application running in the target VM.
    /// All Java threads currently running will be suspended.
    ///
    /// Unlike java.lang.Thread.suspend, suspends of both the virtual machine
    /// and individual threads are counted. Before a thread will run again,
    /// it must be resumed through the VM-level resume command or the
    /// thread-level resume command the same number of times it has been
    /// suspended.
    #[jdwp_command((), 1, 8)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Suspend;

    /// Resumes execution of the application after the suspend command or an
    /// event has stopped it.
    ///
    /// Suspensions of the Virtual Machine and individual threads are counted.
    ///
    /// If a particular thread is suspended n times, it must resumed n times
    /// before it will continue.
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
    pub struct CreateString<'a> {
        /// UTF-8 characters to use in the created string
        string: &'a str,
    }

    /// Retrieve this VM's capabilities.
    ///
    /// The capabilities are returned as booleans, each indicating the presence
    /// or absence of a capability.
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
        /// (that is, can the VM determine if the method or the field was
        /// invented by the compiler?)
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
        /// Base directory used to resolve relative paths in either of the
        /// following lists.
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
    /// The count of references held by the back-end (the reference count) will
    /// be decremented by ref_cnt.
    ///
    /// If thereafter the reference count is less than or equal to zero, the ID
    /// is freed.
    ///
    /// Any back-end resources associated with the freed ID may be freed, and if
    /// garbage collection was disabled for the object, it will be re-enabled.
    ///
    /// The sender of this command promises that no further commands will be
    /// sent referencing a freed ID.
    ///
    /// Use of this command is not required.
    ///
    /// If it is not sent, resources associated with each ID will be freed by
    /// the back-end at some time after the corresponding object is garbage
    /// collected.
    ///
    /// It is most useful to use this command to reduce the load on the back-end
    /// if a very large number of objects has been retrieved from the
    /// back-end (a large array, for example) but may not be garbage
    /// collected any time soon.
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

    /// Tells the target VM to stop sending events. Events are not discarded;
    /// they are held until a subsequent ReleaseEvents command is sent.
    ///
    /// This command is useful to control the number of events sent to the
    /// debugger VM in situations where very large numbers of events are
    /// generated.
    ///
    /// While events are held by the debugger back-end, application execution
    /// may be frozen by the debugger back-end to prevent buffer overflows
    /// on the back end.
    ///
    /// Responses to commands are never held and are not affected by this
    /// command. If events are already being held, this command is ignored.
    #[jdwp_command((), 1, 15)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct HoldEvents;

    /// Tells the target VM to continue sending events.
    ///
    /// This command is used to restore normal activity after a HoldEvents
    /// command.
    ///
    /// If there is no current HoldEvents command in effect, this command is
    /// ignored.
    #[jdwp_command((), 1, 16)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ReleaseEvents;

    /// Retrieve all of this VM's capabilities.
    ///
    /// The capabilities are returned as booleans, each indicating the presence
    /// or absence of a capability.
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
    /// If there are active stack frames in methods of the redefined classes in
    /// the target VM then those active frames continue to run the bytecodes
    /// of the original method. These methods are considered obsolete - see
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
    /// have the `can_add_method` capability to add methods when redefining
    /// classes, or the `can_unrestrictedly_redefine_classes` to redefine
    /// classes in arbitrary ways.
    #[jdwp_command((), 1, 18)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct RedefineClasses<'a> {
        classes: &'a [(ReferenceTypeID, Vec<u8>)],
    }

    /// Set the default stratum. Requires `can_set_default_stratum` capability -
    /// see [CapabilitiesNew].
    #[jdwp_command((), 1, 19)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SetDefaultStratum<'a> {
        /// default stratum, or empty string to use reference type default.
        stratum_id: &'a str,
    }

    /// Returns reference types for all classes currently loaded by the target
    /// VM.
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
        /// The generic signature of the loaded reference type if there is any.
        pub generic_signature: Option<String>,
        /// The current class status
        pub status: ClassStatus,
    }

    /// Returns the number of instances of each reference type in the input
    /// list.
    ///
    /// Only instances that are reachable for the purposes of garbage collection
    /// are counted.
    ///
    /// If a reference type is invalid, eg. it has been unloaded, zero is
    /// returned for its instance count.
    ///
    /// Since JDWP version 1.6. Requires canGetInstanceInfo capability - see
    /// [CapabilitiesNew].
    #[jdwp_command(C::Map<u64>, 1, 21)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct InstanceCounts<C: Coll<Item = ReferenceTypeID>> {
        /// A list of reference type IDs.
        ref_types: C,
    }
}

/// ReferenceType Command Set (2)
pub mod reference_type {
    use std::num::NonZeroU32;

    use super::*;

    /// Returns the JNI signature of a reference type.
    ///
    /// JNI signature formats are described in the Java Native Interface
    /// Specification.
    ///
    /// For primitive classes the returned signature is the signature of the
    /// corresponding primitive type; for example, "I" is returned as the
    /// signature of the class represented by `java.lang.Integer.TYPE`.
    #[jdwp_command(String, 2, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Signature {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the instance of `java.lang.ClassLoader` which loaded a given
    /// reference type.
    ///
    /// If the reference type was loaded by the system class loader, the
    /// returned object ID is null.
    #[jdwp_command(Option<ClassLoaderID>, 2, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ClassLoader {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the modifiers (also known as access flags) for a reference type.
    ///
    /// The returned bit mask contains information on the declaration of the
    /// reference type.
    ///
    /// If the reference type is an array or a primitive class (for example,
    /// `java.lang.Integer.TYPE`), the value of the returned bit mask is
    /// undefined.
    #[jdwp_command(TypeModifiers, 2, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Modifiers {
        ref_type: ReferenceTypeID,
    }

    /// Returns information for each field in a reference type.
    ///
    /// Inherited fields are not included.
    ///
    /// The field list will include any synthetic fields created by the
    /// compiler.
    ///
    /// Fields are returned in the order they occur in the class file.
    #[jdwp_command(Vec<Field>, 2, 4)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Fields {
        ref_type: ReferenceTypeID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct Field {
        /// Field ID
        pub field_id: FieldID,
        /// Name of field
        pub name: String,
        /// JNI Signature of field.
        pub signature: String,
        /// The modifier bit flags (also known as access flags) which provide
        /// additional information on the field declaration.
        ///
        /// Individual flag values are defined in Chapter 4 of The Java™ Virtual
        /// Machine Specification.
        ///
        /// In addition, the 0xf0000000 bit identifies the field as synthetic,
        /// if the synthetic attribute capability is available.
        pub mod_bits: FieldModifiers,
    }

    /// Returns information for each method in a reference type.
    ///
    /// Inherited methods are not included.
    ///
    /// The list of methods will include constructors (identified with the name
    /// "&lt;init>"), the initialization method (identified with the name
    /// "&lt;clinit>") if present, and any synthetic methods created by the
    /// compiler.
    ///
    /// Methods are returned in the order they occur in the class file.
    #[jdwp_command(Vec<Method>, 2, 5)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Methods {
        ref_type: ReferenceTypeID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct Method {
        /// Method ID
        pub method_id: MethodID,
        /// Name of method
        pub name: String,
        /// JNI Signature of method.
        pub signature: String,
        /// The modifier bit flags (also known as access flags) which provide
        /// additional information on the method declaration.
        ///
        /// Individual flag values are defined in Chapter 4 of The Java™ Virtual
        /// Machine Specification.
        ///
        /// In addition, The 0xf0000000 bit identifies the method as synthetic,
        /// if the synthetic attribute capability is available.
        pub mod_bits: MethodModifiers,
    }

    /// Returns the value of one or more static fields of the reference type.
    ///
    /// Each field must be member of the reference type or one of its
    /// superclasses, superinterfaces, or implemented interfaces.
    ///
    /// Access control is not enforced; for example, the values of private
    /// fields can be obtained.
    #[jdwp_command(C::Map<Value>, 2, 6)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct GetValues<C: Coll<Item = FieldID>> {
        /// The reference type ID
        pub ref_type: ReferenceTypeID,
        /// Field IDs of fields to get
        pub fields: C,
    }

    /// Returns the source file name in which a reference type was declared.
    #[jdwp_command(String, 2, 7)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SourceFile {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the classes and interfaces directly nested within this type.
    /// Types further nested within those types are not included.
    #[jdwp_command(Vec<TaggedReferenceTypeID>, 2, 8)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct NestedTypes {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the current status of the reference type.
    ///
    /// The status indicates the extent to which the reference type has been
    /// initialized, as described in section 2.1.6 of The Java™ Virtual Machine
    /// Specification.
    ///
    /// If the class is linked the PREPARED and VERIFIED bits in the returned
    /// status bits will be set.
    ///
    /// If the class is initialized the INITIALIZED bit in the returned status
    /// bits will be set.
    ///
    /// If an error occurred during initialization then the ERROR bit in the
    /// returned status bits will be set.
    ///
    /// The returned status bits are undefined for array types and for primitive
    /// classes (such as java.lang.Integer.TYPE).
    #[jdwp_command(ClassStatus, 2, 9)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Status {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the interfaces declared as implemented by this class.
    ///
    /// Interfaces indirectly implemented (extended by the implemented interface
    /// or implemented by a superclass) are not included.
    #[jdwp_command(Vec<InterfaceID>, 2, 10)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Interfaces {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the class object corresponding to this type.
    #[jdwp_command(ClassObjectID, 2, 11)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ClassObject {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the value of the SourceDebugExtension attribute.
    ///
    /// Since JDWP version 1.4. Requires canGetSourceDebugExtension capability -
    /// see [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    #[jdwp_command(String, 2, 12)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SourceDebugExtension {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    /// Returns the JNI signature of a reference type along with the generic
    /// signature if there is one.
    ///
    /// Generic signatures are described in the signature attribute section in
    /// The Java™ Virtual Machine Specification.
    ///
    /// Since JDWP version 1.5.
    #[jdwp_command(2, 13)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SignatureWithGeneric {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct SignatureWithGenericReply {
        /// The JNI signature for the reference type.
        pub signature: String,
        /// The generic signature for the reference type or an empty string if
        /// there is none.
        pub generic_signature: String,
    }

    /// Returns information, including the generic signature if any, for each
    /// field in a reference type.
    ///
    /// Inherited fields are not included.
    ///
    /// The field list will include any synthetic fields created by the
    /// compiler.
    ///
    /// Fields are returned in the order they occur in the class file.
    ///
    /// Generic signatures are described in the signature attribute section in
    /// The Java™ Virtual Machine Specification.
    ///
    /// Since JDWP version 1.5.

    #[jdwp_command(Vec<FieldWithGeneric>, 2, 14)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct FieldsWithGeneric {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct FieldWithGeneric {
        /// The field ID
        pub field_id: FieldID,
        /// The name of the field
        pub name: String,
        /// The JNI signature of the field
        pub signature: String,
        /// The generic signature of the field if there is one
        pub generic_signature: Option<String>,
        /// The modifier bit flags (also known as access flags) which provide
        /// additional information on the field declaration.
        ///
        /// Individual flag values are defined in Chapter 4 of The Java™ Virtual
        /// Machine Specification.
        ///
        /// In addition, the 0xf0000000 bit identifies the field as synthetic,
        /// if the synthetic attribute capability is available.
        pub mod_bits: FieldModifiers,
    }

    /// Returns information, including the generic signature if any, for each
    /// method in a reference type. Inherited methodss are not included.
    /// The list of methods will include constructors (identified with the name
    /// "&lt;init>"), the initialization method (identified with the name
    /// "&lt;clinit>") if present, and any synthetic methods created by the
    /// compiler. Methods are returned in the order they occur in the class
    /// file.
    ///
    /// Generic signatures are described in the signature attribute section in
    /// The Java™ Virtual Machine Specification.
    ///
    /// Since JDWP version 1.5.
    #[jdwp_command(Vec<MethodWithGeneric>, 2, 15)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct MethodsWithGeneric {
        /// The reference type ID
        ref_type: ReferenceTypeID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct MethodWithGeneric {
        /// The method ID
        pub method_id: MethodID,
        /// The name of the method
        pub name: String,
        /// The JNI signature of the method
        pub signature: String,
        /// The generic signature of the method, or an empty string if there is
        /// none
        pub generic_signature: String,
        /// The modifier bit flags (also known as access flags) which provide
        /// additional information on the method declaration.
        ///
        /// Individual flag values are defined in Chapter 4 of The Java™ Virtual
        /// Machine Specification.
        ///
        /// In addition, the 0xf0000000 bit identifies the method as synthetic,
        /// if the synthetic attribute capability is available.
        pub mod_bits: MethodModifiers,
    }

    /// Returns instances of this reference type.
    ///
    /// Only instances that are reachable for the purposes of garbage collection
    /// are returned.
    #[jdwp_command(Vec<TaggedObjectID>, 2, 16)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Instances {
        /// The reference type ID
        ref_type: ReferenceTypeID,
        /// Maximum number of instances to return.
        max_instances: InstanceLimit,
    }

    #[derive(Debug, Clone)]
    pub enum InstanceLimit {
        All,
        Limit(NonZeroU32),
    }

    impl InstanceLimit {
        /// A shorthand for `InstanceLimit::Limit`.
        ///
        /// # Panics
        /// Panics if `limit` is zero.
        pub fn limit(limit: u32) -> Self {
            InstanceLimit::Limit(NonZeroU32::new(limit).expect("Instance limit was zero"))
        }
    }

    impl JdwpWritable for InstanceLimit {
        fn write<W: io::Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
            match self {
                InstanceLimit::All => 0u32.write(write),
                InstanceLimit::Limit(limit) => limit.get().write(write),
            }
        }
    }

    /// Returns the class object corresponding to this type.
    #[jdwp_command(2, 17)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ClassFileVersion {
        /// The class
        ref_type: ReferenceTypeID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct ClassFileVersionReply {
        /// Major version number
        pub major_version: u32,
        /// Minor version number
        pub minor_version: u32,
    }

    /// Return the raw bytes of the constant pool in the format of the
    /// constant_pool item of the Class File Format in The Java™ Virtual Machine
    /// Specification.
    ///
    /// Since JDWP version 1.6. Requires canGetConstantPool capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    #[jdwp_command(2, 18)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ConstantPool {
        /// The class
        ref_type: ReferenceTypeID,
    }

    #[derive(JdwpReadable)]
    pub struct ConstantPoolReply {
        /// Total number of constant pool entries plus one.
        ///
        /// This corresponds to the constant_pool_count item of the Class File
        /// Format in The Java™ Virtual Machine Specification.
        pub count: u32,
        /// Raw bytes of the constant pool
        pub bytes: Vec<u8>,
    }

    // special debug so that trace logs dont take a quadrillion lines
    impl Debug for ConstantPoolReply {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let hex_bytes = self
                .bytes
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();

            struct Unquoted(String);

            impl Debug for Unquoted {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(&self.0)
                }
            }

            f.debug_struct("ConstantPoolReply")
                .field("count", &self.count)
                .field("bytes", &Unquoted(hex_bytes))
                .finish()
        }
    }
}

/// ClassType Command Set (3)
pub mod class_type {
    use super::*;

    /// Returns the immediate superclass of a class.
    ///
    /// The return is null if the class is java.lang.Object.
    #[jdwp_command(Option<ClassID>, 3, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Superclass {
        /// The class type ID.
        class_id: ClassID,
    }

    /// Sets the value of one or more static fields.
    ///
    /// Each field must be member of the class type or one of its superclasses,
    /// superinterfaces, or implemented interfaces.
    ///
    /// Access control is not enforced; for example, the values of private
    /// fields can be set.
    ///
    /// Final fields cannot be set.
    ///
    /// For primitive values, the value's type must match the field's type
    /// exactly.
    ///
    /// For object values, there must exist a widening reference conversion from
    /// the value's type to thefield's type and the field's type must be
    /// loaded.
    #[jdwp_command((), 3, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SetValues<'a> {
        /// The class type ID.
        class_id: ClassID,
        /// Fields to set and their values.
        values: &'a [(FieldID, UntaggedValue)],
    }

    /// Invokes a static method. The method must be member of the class type or
    /// one of its superclasses, superinterfaces, or implemented interfaces.
    /// Access control is not enforced; for example, private methods can be
    /// invoked.
    ///
    /// The method invocation will occur in the specified thread. Method
    /// invocation can occur only if the specified thread has been suspended
    /// by an event. Method invocation is not supported when the target VM
    /// has been suspended by the front-end.
    ///
    /// The specified method is invoked with the arguments in the specified
    /// argument list. The method invocation is synchronous; the reply
    /// packet is not sent until the invoked method returns in the target
    /// VM. The return value (possibly the void value) is included in the
    /// reply packet. If the invoked method throws an exception, the
    /// exception object ID is set in the reply packet; otherwise, the
    /// exception object ID is null.
    ///
    /// For primitive arguments, the argument value's type must match the
    /// argument's type exactly. For object arguments, there must exist a
    /// widening reference conversion from the argument value's type to the
    /// argument's type and the argument's type must be loaded.
    ///
    /// By default, all threads in the target VM are resumed while the method is
    /// being invoked if they were previously suspended by an event or by
    /// command. This is done to prevent the deadlocks that will occur if
    /// any of the threads own monitors that will be needed by the invoked
    /// method. It is possible that breakpoints or other events might occur
    /// during the invocation. Note, however, that this implicit resume acts
    /// exactly like the ThreadReference
    /// [resume](super::thread_reference::Resume) command, so if the
    /// thread's suspend count is greater than 1, it will remain in a suspended
    /// state during the invocation. By default, when the invocation completes,
    /// all threads in the target VM are suspended, regardless their state
    /// before the invocation.
    ///
    /// The resumption of other threads during the invoke can be prevented by
    /// specifying the
    /// [INVOKE_SINGLE_THREADED](crate::spec::InvokeOptions::SINGLE_THREADED)
    /// bit flag in the options field; however, there is no protection
    /// against or recovery from the deadlocks described above, so this
    /// option should be used with great caution. Only the specified thread
    /// will be resumed (as described for all threads above). Upon
    /// completion of a single threaded invoke, the invoking thread will be
    /// suspended once again. Note that any threads started during the
    /// single threaded invocation will not be suspended when the invocation
    /// completes.
    ///
    /// If the target VM is disconnected during the invoke (for example, through
    /// the VirtualMachine [dispose](super::virtual_machine::Dispose)
    /// command) the method invocation continues.

    #[jdwp_command(3, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct InvokeMethod<'a> {
        /// The class type ID
        class_id: ClassID,
        /// The thread in which to invoke
        thread_id: ThreadID,
        /// The method to invoke
        method_id: MethodID,
        /// Arguments to the method
        arguments: &'a [Value],
        // Invocation options
        options: InvokeOptions,
    }

    #[jdwp_command(3, 4)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct NewInstance<'a> {
        /// The class type ID.
        class_id: ClassID,
        /// The thread in which to invoke the constructor.
        thread_id: ThreadID,
        /// The constructor to invoke.
        method_id: MethodID,
        /// Arguments for the constructor method.
        arguments: &'a [Value],
        // Constructor invocation options
        options: InvokeOptions,
    }

    #[derive(Debug)]
    pub enum NewInstanceReply {
        /// The newly created object.
        NewObject(TaggedObjectID),
        /// The thrown exception.
        Exception(TaggedObjectID),
    }

    // better types everyone
    impl JdwpReadable for NewInstanceReply {
        fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
            let new_object = Option::<TaggedObjectID>::read(read)?;
            let exception = Option::<TaggedObjectID>::read(read)?;

            match (new_object, exception) {
                (Some(new_object), None) => Ok(NewInstanceReply::NewObject(new_object)),
                (None, Some(exception)) => Ok(NewInstanceReply::Exception(exception)),
                _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
            }
        }
    }
}

/// ArrayType Command Set (4)
pub mod array_type {
    use super::*;

    /// Creates a new array object of this type with a given length.
    #[jdwp_command(4, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct NewInstance {
        /// The array type of the new instance
        array_type_id: ArrayTypeID,
        /// The length of the array
        length: u32,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct NewInstanceReply {
        // should always be Tag::Array
        _tag: Tag,
        /// The newly created array object
        pub new_array: ArrayID,
    }

    impl Deref for NewInstanceReply {
        type Target = ArrayID;

        fn deref(&self) -> &Self::Target {
            &self.new_array
        }
    }
}

/// InterfaceType Command Set (5)
pub mod interface_type {
    use super::*;

    /// Invokes a static method. The method must not be a static initializer.
    /// The method must be a member of the interface type.
    ///
    /// Since JDWP version 1.8
    ///
    /// The method invocation will occur in the specified thread. Method
    /// invocation can occur only if the specified thread has been suspended by
    /// an event. Method invocation is not supported when the target VM has been
    /// suspended by the front-end.
    ///
    /// The specified method is invoked with the arguments in the specified
    /// argument list. The method invocation is synchronous; the reply packet is
    /// not sent until the invoked method returns in the target VM. The return
    /// value (possibly the void value) is included in the reply packet. If the
    /// invoked method throws an exception, the exception object ID is set in
    /// the reply packet; otherwise, the exception object ID is null.
    ///
    /// For primitive arguments, the argument value's type must match the
    /// argument's type exactly. For object arguments, there must exist a
    /// widening reference conversion from the argument value's type to the
    /// argument's type and the argument's type must be loaded.
    ///
    /// By default, all threads in the target VM are resumed while the method is
    /// being invoked if they were previously suspended by an event or by a
    /// command. This is done to prevent the deadlocks that will occur if any of
    /// the threads own monitors that will be needed by the invoked method. It
    /// is possible that breakpoints or other events might occur during the
    /// invocation. Note, however, that this implicit resume acts exactly like
    /// the ThreadReference resume command, so if the thread's suspend count is
    /// greater than 1, it will remain in a suspended state during the
    /// invocation. By default, when the invocation completes, all threads in
    /// the target VM are suspended, regardless their state before the
    /// invocation.
    ///
    /// The resumption of other threads during the invoke can be prevented by
    /// specifying the SINGLE_THREADED bit flag in the options field;
    /// however, there is no protection against or recovery from the deadlocks
    /// described above, so this option should be used with great caution. Only
    /// the specified thread will be resumed (as described for all threads
    /// above). Upon completion of a single threaded invoke, the invoking thread
    /// will be suspended once again. Note that any threads started during the
    /// single threaded invocation will not be suspended when the invocation
    /// completes.

    // If the target VM is disconnected during the invoke (for example, through the VirtualMachine
    // [Dispose](super::virtual_machine::Dispose) command) the method invocation continues.
    #[jdwp_command(5, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct InvokeMethod<'a> {
        /// The interface type ID
        interface_id: InterfaceID,
        /// The thread in which to invoke
        thread_id: ThreadID,
        /// The method to invoke
        method_id: MethodID,
        /// The argument values
        arguments: &'a [Value],
        /// Invocation options
        options: InvokeOptions,
    }
}

/// Method Command Set (6)
pub mod method {
    use super::*;

    /// Returns line number information for the method, if present.
    ///
    /// The line table maps source line numbers to the initial code index of the
    /// line.
    ///
    /// The line table is ordered by code index (from lowest to highest).
    ///
    /// The line number information is constant unless a new class definition is
    /// installed using
    /// [RedefineClasses](super::virtual_machine::RedefineClasses).
    #[jdwp_command(6, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct LineTable {
        /// The class.
        reference_type_id: ReferenceTypeID,
        /// The method.
        method_id: MethodID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct LineTableReply {
        /// Lowest valid code index for the method, >=0, or -1 if the method is
        /// native
        pub start: i64,
        /// Highest valid code index for the method, >=0, or -1 if the method is
        /// native
        pub end: i64,
        /// The entries of the line table for this method.
        pub lines: Vec<Line>,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct Line {
        /// Initial code index of the line, start <= lineCodeIndex < end
        pub line_code_index: u64,
        /// Line number.
        pub line_number: u32,
    }

    /// Returns variable information for the method.
    ///
    /// The variable table includes arguments and locals declared within the
    /// method. For instance methods, the "this" reference is included in
    /// the table. Also, synthetic variables may be present.
    #[jdwp_command(6, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct VariableTable {
        /// The class.
        reference_type_id: ReferenceTypeID,
        /// The method.
        method_id: MethodID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct VariableTableReply {
        /// The number of words in the frame used by arguments. Eight-byte
        /// arguments use two words; all others use one.
        pub arg_cnt: u32,
        /// The variables.
        pub variables: Vec<Variable>,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct Variable {
        /// First code index at which the variable is visible.
        ///
        /// Used in conjunction with length. The variable can be get or set only
        /// when the current codeIndex <= current frame code index < codeIndex +
        /// length
        pub code_index: u64,
        /// The variable's name.
        pub name: String,
        /// The variable type's JNI signature.
        pub signature: String,
        /// Unsigned value used in conjunction with codeIndex.
        ///
        /// The variable can be get or set only when the current codeIndex <=
        /// current frame code index < code index + length
        pub length: u32,
        /// The local variable's index in its frame
        pub slot: u32,
    }

    /// Retrieve the method's bytecodes as defined in The Java™ Virtual Machine
    /// Specification.
    ///
    /// Requires `canGetBytecodes` capability - see
    /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    #[jdwp_command(Vec<u8>, 6, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Bytecodes {
        /// The class.
        reference_type_id: ReferenceTypeID,
        /// The method.
        method_id: MethodID,
    }

    /// Determine if this method is obsolete.
    ///
    /// A method is obsolete if it has been replaced by a non-equivalent method
    /// using the [RedefineClasses](super::virtual_machine::RedefineClasses)
    /// command. The original and redefined methods are considered equivalent if
    /// their bytecodes are the same except for indices into the constant pool
    /// and the referenced constants are equal.
    #[jdwp_command(bool, 6, 4)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct IsObsolete {
        /// The class.
        reference_type_id: ReferenceTypeID,
        /// The method.
        method_id: MethodID,
    }

    /// Returns variable information for the method, including generic
    /// signatures for the variables.
    ///
    /// The variable table includes arguments and locals declared within the
    /// method. For instance methods, the "this" reference is included in
    /// the table. Also, synthetic variables may be present. Generic
    /// signatures are described in the signature attribute section in The
    /// Java™ Virtual Machine Specification.
    ///
    /// Since JDWP version 1.5.
    #[jdwp_command(6, 5)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct VariableTableWithGeneric {
        /// The class.
        reference_type_id: ReferenceTypeID,
        /// The method.
        method_id: MethodID,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct VariableTableWithGenericReply {
        /// The number of words in the frame used by arguments. Eight-byte
        /// arguments use two words; all others use one.
        pub arg_cnt: u32,
        /// The variables.
        pub variables: Vec<VariableWithGeneric>,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct VariableWithGeneric {
        /// First code index at which the variable is visible.
        ///
        /// Used in conjunction with length. The variable can be get or set only
        /// when the current codeIndex <= current frame code index < codeIndex +
        /// length
        pub code_index: u64,
        /// The variable's name.
        pub name: String,
        /// The variable type's JNI signature.
        pub signature: String,
        /// The variable type's generic signature or an empty string if there is
        /// none.
        pub generic_signature: String,
        /// Unsigned value used in conjunction with codeIndex.
        ///
        /// The variable can be get or set only when the current codeIndex <=
        /// current frame code index < code index + length
        pub length: u32,
        /// The local variable's index in its frame
        pub slot: u32,
    }
}

/// Field Command Set (8)
pub mod field {}

/// ObjectReference Command Set (9)
pub mod object_reference {
    use super::*;

    /// Returns the runtime type of the object. The runtime type will be a class
    /// or an array.
    #[jdwp_command(TaggedReferenceTypeID, 9, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ReferenceType {
        /// The object ID
        object: ObjectID,
    }

    /// Returns the value of one or more instance fields.
    ///
    /// Each field must be member of the object's type or one of its
    /// superclasses, superinterfaces, or implemented interfaces. Access
    /// control is not enforced; for example, the values of private fields
    /// can be obtained.
    #[jdwp_command(C::Map<Value>, 9, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct GetValues<C: Coll<Item = FieldID>> {
        /// The object ID
        object: ObjectID,
        /// Fields to get
        fields: C,
    }

    /// Sets the value of one or more instance fields.
    ///
    /// Each field must be member of the object's type or one of its
    /// superclasses, superinterfaces, or implemented interfaces. Access
    /// control is not enforced; for example, the values of private fields
    /// can be set. For primitive values, the value's type must match the
    /// field's type exactly. For object values, there must be a widening
    /// reference conversion from the value's type to the field's type and
    /// the field's type must be loaded.
    #[jdwp_command((), 9, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SetValues<'a> {
        /// The object ID
        object: ObjectID,
        /// Fields and the values to set them to
        fields: &'a [(FieldID, UntaggedValue)],
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
        /// The threads that are waiting for the monitor 0 if there is no
        /// current owner
        pub waiters: Vec<ThreadID>,
    }

    /// Invokes a instance method.
    ///
    /// The method must be member of the object's type or one of its
    /// superclasses, superinterfaces, or implemented interfaces. Access
    /// control is not enforced; for example, private methods can be
    /// invoked.
    ///
    /// The method invocation will occur in the specified thread. Method
    /// invocation can occur only if the specified thread has been suspended
    /// by an event. Method invocation is not supported when the target VM
    /// has been suspended by the front-end.
    ///
    /// The specified method is invoked with the arguments in the specified
    /// argument list. The method invocation is synchronous; the reply
    /// packet is not sent until the invoked method returns in the target
    /// VM. The return value (possibly the void value) is included in the
    /// reply packet.
    ///
    /// For primitive arguments, the argument value's type must match the
    /// argument's type exactly. For object arguments, there must be a
    /// widening reference conversion from the argument value's type to the
    /// argument's type and the argument's type must be loaded.
    ///
    /// By default, all threads in the target VM are resumed while the method is
    /// being invoked if they were previously suspended by an event or by a
    /// command. This is done to prevent the deadlocks that will occur if
    /// any of the threads own monitors that will be needed by the invoked
    /// method. It is possible that breakpoints or other events might occur
    /// during the invocation. Note, however, that this implicit resume acts
    /// exactly like the ThreadReference resume command, so if the thread's
    /// suspend count is greater than 1, it will remain in a suspended state
    /// during the invocation. By default, when the invocation completes,
    /// all threads in the target VM are suspended, regardless their state
    /// before the invocation.
    ///
    /// The resumption of other threads during the invoke can be prevented by
    /// specifying the INVOKE_SINGLE_THREADED bit flag in the options field;
    /// however, there is no protection against or recovery from the deadlocks
    /// described above, so this option should be used with great caution. Only
    /// the specified thread will be resumed (as described for all threads
    /// above). Upon completion of a single threaded invoke, the invoking
    /// thread will be suspended once again. Note that any threads started
    /// during the single threaded invocation will not be suspended when the
    /// invocation completes.
    ///
    /// If the target VM is disconnected during the invoke (for example, through
    /// the VirtualMachine dispose command) the method invocation continues.
    #[jdwp_command(9, 6)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct InvokeMethod<'a> {
        /// The object ID
        object: ObjectID,
        /// The thread in which to invoke
        thread: ThreadID,
        /// The method to invoke
        method: (ClassID, MethodID),
        /// The arguments
        arguments: &'a [Value],
        /// Invocation options
        options: InvokeOptions,
    }

    /// Prevents garbage collection for the given object.
    ///
    /// By default all objects in back-end replies may be collected at any time
    /// the target VM is running. A call to this command guarantees that the
    /// object will not be collected. The [EnableCollection] command can be
    /// used to allow collection once again.
    ///
    /// Note that while the target VM is suspended, no garbage collection will
    /// occur because all threads are suspended. The typical examination of
    /// variables, fields, and arrays during the suspension is safe without
    /// explicitly disabling garbage collection.
    ///
    /// This method should be used sparingly, as it alters the pattern of
    /// garbage collection in the target VM and, consequently, may result in
    /// application behavior under the debugger that differs from its
    /// non-debugged behavior.
    #[jdwp_command((), 9, 7)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct DisableCollection {
        /// The object ID
        object: ObjectID,
    }

    /// Permits garbage collection for this object.
    ///
    /// By default all objects returned by JDWP may become unreachable in the
    /// target VM, and hence may be garbage collected. A call to this
    /// command is necessary only if garbage collection was previously
    /// disabled with the [DisableCollection] command.
    #[jdwp_command((), 9, 8)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct EnableCollection {
        /// The object ID
        object: ObjectID,
    }

    /// Determines whether an object has been garbage collected in the target
    /// VM.
    #[jdwp_command(bool, 9, 9)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct IsCollected {
        /// The object ID
        object: ObjectID,
    }

    /// Returns objects that directly reference this object. Only objects that
    /// are reachable for the purposes of garbage collection are returned.
    /// Note that an object can also be referenced in other ways, such as
    /// from a local variable in a stack frame, or from a JNI global
    /// reference. Such non-object referrers are not returned by this
    /// command.
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
        /// Maximum number of referring objects to return. Must be non-negative.
        /// If zero, all referring objects are returned.
        max_referrers: u32,
    }
}

/// StringReference Command Set (10)
pub mod string_reference {
    use super::*;

    /// Returns the characters contained in the string.
    #[jdwp_command(String, 10, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Value {
        /// The String object ID
        string_object: StringID,
    }
}

/// ThreadReference Command Set (11)
pub mod thread_reference {
    use super::*;

    /// Returns the thread name.
    #[jdwp_command(String, 11, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Name {
        /// The thread object ID.
        pub thread: ThreadID,
    }

    /// Suspends the thread.
    ///
    /// Unlike `java.lang.Thread.suspend()`, suspends of both the virtual
    /// machine and individual threads are counted. Before a thread will run
    /// again, it must be resumed the same number of times it has been
    /// suspended.
    ///
    /// Suspending single threads with command has the same dangers
    /// `java.lang.Thread.suspend()`. If the suspended thread holds a monitor
    /// needed by another running thread, deadlock is possible in the target
    /// VM (at least until the suspended thread is resumed again).
    ///
    /// The suspended thread is guaranteed to remain suspended until resumed
    /// through one of the JDI resume methods mentioned above; the
    /// application in the target VM cannot resume the suspended thread
    /// through `java.lang.Thread.resume()`.
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
    /// If this thread was not previously suspended by the front-end, calling
    /// this command has no effect. Otherwise, the count of pending suspends
    /// on this thread is decremented. If it is decremented to 0, the thread
    /// will continue to execute.
    #[jdwp_command((), 11, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Resume {
        /// The thread object ID.
        pub thread: ThreadID,
    }

    /// Returns the current status of a thread.
    ///
    /// The thread status reply indicates the thread status the last time it was
    /// running. the suspend status provides information on the thread's
    /// suspension, if any.
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
    /// The sequence of frames starts with the currently executing frame,
    /// followed by its caller, and so on. The thread must be suspended, and
    /// the returned frameID is valid only while the thread is suspended.
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

    /// A nice readable enum to be used in place of raw `i32` with a special
    /// meaning for -1.
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
    /// The thread must be suspended, and the returned count is valid only while
    /// the thread is suspended.
    ///
    /// Returns [ThreadNotSuspended](crate::spec::ErrorCode::ThreadNotSuspended)
    /// if not suspended.
    #[jdwp_command(u32, 11, 7)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct FrameCount {
        /// The thread object ID.
        pub thread: ThreadID,
    }

    /// Returns the objects whose monitors have been entered by this thread.
    ///
    /// The thread must be suspended, and the returned information is relevant
    /// only while the thread is suspended.
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
    /// The thread may be waiting to enter a monitor, or it may be waiting, via
    /// the `java.lang.Object.wait` method, for another thread to invoke the
    /// notify method. The thread must be suspended, and the returned
    /// information is relevant only while the thread is suspended.
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
        /// This object must be an instance of `java.lang.Throwable` or a
        /// subclass
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

    /// Returns monitor objects owned by the thread, along with stack depth at
    /// which the monitor was acquired.
    ///
    /// Stack depth can be unknown (e.g., for monitors acquired by JNI
    /// MonitorEnter). The thread must be suspended, and the returned
    /// information is relevant only while the thread is suspended.
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
    /// The method which will return early is referred to as the called method.
    /// The called method is the current method (as defined by the Frames
    /// section in The Java™ Virtual Machine Specification) for the
    /// specified thread at the time this command is received.
    ///
    /// The specified thread must be suspended. The return occurs when execution
    /// of Java programming language code is resumed on this thread. Between
    /// sending this command and resumption of thread execution, the state
    /// of the stack is undefined.
    ///
    /// No further instructions are executed in the called method. Specifically,
    /// finally blocks are not executed. Note: this can cause inconsistent
    /// states in the application.
    ///
    /// A lock acquired by calling the called method (if it is a synchronized
    /// method) and locks acquired by entering synchronized blocks within the
    /// called method are released. Note: this does not apply to JNI locks
    /// or java.util.concurrent.locks locks.
    ///
    /// Events, such as [MethodExit](super::event::Event::MethodExit), are
    /// generated as they would be in a normal return.
    ///
    /// The called method must be a non-native Java programming language method.
    /// Forcing return on a thread with only one frame on the stack causes the
    /// thread to exit when resumed.
    ///
    /// For void methods, the value must be a void value. For methods that
    /// return primitive values, the value's type must match the return type
    /// exactly. For object values, there must be a widening reference
    /// conversion from the value's type to the return type type and the
    /// return type must be loaded.
    ///
    /// Since JDWP version 1.6. Requires `can_force_early_return` capability -
    /// see [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
    #[jdwp_command((), 11, 14)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ForceEarlyReturn {
        /// The thread object ID.
        pub thread: ThreadID,
        /// The value to return.
        pub value: Value,
    }
}

/// ThreadGroupReference Command Set (12)
pub mod thread_group_reference {
    use super::*;

    /// Returns the thread group name.
    #[jdwp_command(String, 12, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Name {
        /// The thread group object ID
        group: ThreadGroupID,
    }

    /// Returns the thread group, if any, which contains a given thread group.
    #[jdwp_command(Option<ThreadGroupID>, 12, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
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
    #[derive(Debug, Clone, JdwpWritable)]
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
}

/// ArrayReference Command Set (13)
pub mod array_reference {
    use super::*;

    /// Returns the number of components in a given array.
    #[jdwp_command(u32, 13, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Length {
        /// The array object ID
        array_id: ArrayID,
    }

    /// Returns a range of array components.
    ///
    /// The specified range must be within the bounds of the array.
    #[jdwp_command(ArrayRegion, 13, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct GetValues {
        /// The array object ID
        array_id: ArrayID,
        /// The first index to retrieve
        first_index: u32,
        /// The number of components to retrieve
        length: u32,
    }

    /// Sets a range of array components.
    ///
    /// The specified range must be within the bounds of the array.
    ///
    /// For primitive values, each value's type must match the array component
    /// type exactly.
    ///
    /// For object values, there must be a widening reference conversion from
    /// the value's type to the array component type and the array component
    /// type must be loaded.
    #[jdwp_command((), 13, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SetValues<'a, V: JdwpValue> {
        /// The array object ID
        array_id: ArrayID,
        /// The first index to set
        first_index: u32,
        /// Values to set
        values: &'a [V],
    }
}

/// ClassLoaderReference Command Set (14)
pub mod class_loader_reference {
    use super::*;

    /// Returns a list of all classes which this class loader has been requested
    /// to load.
    ///
    /// This class loader is considered to be an initiating class loader for
    /// each class in the returned list. The list contains each reference
    /// type defined by this loader and any types for which loading was
    /// delegated by this class loader to another class loader.
    ///
    /// The visible class list has useful properties with respect to the type
    /// namespace.
    ///
    /// A particular type name will occur at most once in the list.
    ///
    /// Each field or variable declared with that type name in a class defined
    /// by this class loader must be resolved to that single type.
    ///
    /// No ordering of the returned list is guaranteed.
    #[jdwp_command(Vec<TaggedReferenceTypeID>, 14, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct VisibleClasses {
        /// The class loader object ID
        class_loader_id: ClassLoaderID,
    }
}

/// EventRequest Command Set (15)
pub mod event_request {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, JdwpWritable)]
    #[repr(u8)]
    pub enum Modifier<'a> {
        /// Limit the requested event to be reported at most once after a given
        /// number of occurrences.
        ///
        /// The event is not reported the first count - 1 times this filter is
        /// reached.
        ///
        /// To request a one-off event, call this method with a count of 1.
        ///
        /// Once the count reaches 0, any subsequent filters in this request are
        /// applied.
        ///
        /// If none of those filters cause the event to be suppressed, the event
        /// is reported.
        ///
        /// Otherwise, the event is not reported.
        ///
        /// In either case subsequent events are never reported for this
        /// request.
        ///
        /// This modifier can be used with any event kind.
        Count(
            /// Count before event. One for one-off
            i32,
        ) = ModifierKind::Count as u8,

        /// Conditional on expression
        Conditional {
            /// For the future
            expr_id: i32,
        } = ModifierKind::Conditional as u8,

        /// Restricts reported events to those in the given thread.
        /// This modifier can be used with any event kind except for class
        /// unload.
        ThreadOnly(
            /// Required thread
            ThreadID,
        ) = ModifierKind::ThreadOnly as u8,

        /// For class prepare events, restricts the events generated by this
        /// request to be the preparation of the given reference type
        /// and any subtypes.
        ///
        /// For monitor wait and waited events, restricts the events generated
        /// by this request to those whose monitor object is of the
        /// given reference type or any of its subtypes.
        ///
        /// For other events, restricts the events generated by this request to
        /// those whose location is in the given reference type or any of its
        /// subtypes.
        ///
        /// An event will be generated for any location in a reference type that
        /// can be safely cast to the given reference type.
        ///
        /// This modifier can be used with any event kind except class unload,
        /// thread start, and thread end.
        ClassOnly(
            /// Required class
            ReferenceTypeID,
        ) = ModifierKind::ClassOnly as u8,

        /// Restricts reported events to those for classes whose name matches
        /// the given restricted regular expression.
        ///
        /// For class prepare events, the prepared class name is matched.
        ///
        /// For class unload events, the unloaded class name is matched.
        ///
        /// For monitor wait and waited events, the name of the class of the
        /// monitor object is matched.
        ///
        /// For other events, the class name of the event's location is matched.
        ///
        /// This modifier can be used with any event kind except thread start
        /// and thread end.
        ClassMatch(
            /// Required class pattern.
            ///
            /// Matches are limited to exact matches of the given class pattern
            /// and matches of patterns that begin or end with `*`;
            /// for example, `*.Foo` or `java.*`.
            &'a str,
        ) = ModifierKind::ClassMatch as u8,

        /// Restricts reported events to those for classes whose name does not
        /// match the given restricted regular expression.
        ///
        /// For class prepare events, the prepared class name is matched.
        ///
        /// For class unload events, the unloaded class name is matched.
        ///
        /// For monitor wait and waited events, the name of the class of the
        /// monitor object is matched.
        ///
        /// For other events, the class name of the event's location is matched.
        ///
        /// This modifier can be used with any event kind except thread start
        /// and thread end.
        ClassExclude(
            /// Disallowed class pattern.
            ///
            /// Matches are limited to exact matches of the given class pattern
            /// and matches of patterns that begin or end with `*`;
            /// for example, `*.Foo` or `java.*`.
            &'a str,
        ) = ModifierKind::ClassExclude as u8,

        /// Restricts reported events to those that occur at the given location.
        ///
        /// This modifier can be used with breakpoint, field access, field
        /// modification, step, and exception event kinds.
        LocationOnly(
            /// Required location
            Location,
        ) = ModifierKind::LocationOnly as u8,

        /// Restricts reported exceptions by their class and whether they are
        /// caught or uncaught.
        ///
        /// This modifier can be used with exception event kinds only.
        ExceptionOnly {
            /// Exception to report. `None` means report exceptions of all
            /// types.
            ///
            /// A non-null type restricts the reported exception events to
            /// exceptions of the given type or any of its subtypes.
            exception: Option<ReferenceTypeID>,
            /// Report caught exceptions
            uncaught: bool,
            /// Report uncaught exceptions.
            ///
            /// Note that it is not always possible to determine whether an
            /// exception is caught or uncaught at the time it is thrown.
            ///
            /// See the exception event catch location under composite events
            /// for more information.
            caught: bool,
        } = ModifierKind::ExceptionOnly as u8,

        /// Restricts reported events to those that occur for a given field.
        ///
        /// This modifier can be used with field access and field modification
        /// event kinds only.
        FieldOnly(
            /// Type in which field is declared
            ReferenceTypeID,
            /// Required field
            FieldID,
        ) = ModifierKind::FieldOnly as u8,

        /// Restricts reported step events to those which satisfy depth and size
        /// constraints.
        ///
        /// This modifier can be used with step event kinds only.
        Step(
            /// Thread in which to step
            ThreadID,
            /// Size of each step
            StepSize,
            /// Relative call stack limit
            StepDepth,
        ) = ModifierKind::Step as u8,

        /// Restricts reported events to those whose active 'this' object is the
        /// given object.
        ///
        /// Match value is the null object for static methods.
        ///
        /// This modifier can be used with any event kind except class prepare,
        /// class unload, thread start, and thread end.
        ///
        /// Introduced in JDWP version 1.4.
        InstanceOnly(
            /// Required 'this' object
            ObjectID,
        ) = ModifierKind::InstanceOnly as u8,

        /// Restricts reported class prepare events to those for reference types
        /// which have a source name which matches the given restricted regular
        /// expression.
        ///
        /// The source names are determined by the reference type's
        /// SourceDebugExtension.
        ///
        /// This modifier can only be used with class prepare events.
        ///
        /// Since JDWP version 1.6.
        ///
        /// Requires the `can_use_source_name_filters` capability - see
        /// [CapabilitiesNew](crate::spec::virtual_machine::CapabilitiesNew).
        SourceNameMatch(
            /// Required source name pattern.
            /// Matches are limited to exact matches of the given pattern and
            /// matches of patterns that begin or end with `*`; for example,
            /// `*.Foo` or `java.*`
            &'a str,
        ) = ModifierKind::SourceNameMatch as u8,
    }

    /// Set an event request.
    ///
    /// When the event described by this request occurs, an event is sent from
    /// the target VM.
    ///
    /// If an event occurs that has not been requested then it is not sent from
    /// the target VM.
    ///
    /// The two exceptions to this are the VM Start Event and the VM Death Event
    /// which are automatically generated events - see
    /// [Composite](super::event::Composite) command for further details.
    #[jdwp_command(RequestID, 15, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Set<'a> {
        /// Event kind to request. Some events may require a capability in order
        /// to be requested.
        event_kind: EventKind,
        /// What threads are suspended when this event occurs?
        ///
        /// Note that the order of events and command replies accurately
        /// reflects the order in which threads are suspended and
        /// resumed.
        ///
        /// For example, if a VM-wide resume is processed before an event occurs
        /// which suspends the VM, the reply to the resume command will be
        /// written to the transport before the suspending event.
        suspend_policy: SuspendPolicy,
        /// Constraints used to control the number of generated events.
        ///
        /// Modifiers specify additional tests that an event must satisfy before
        /// it is placed in the event queue.
        ///
        /// Events are filtered by applying each modifier to an event in the
        /// order they are specified in this collection. Only events
        /// that satisfy all modifiers are reported.
        ///
        /// An empty list means there are no modifiers in the request.
        ///
        /// Filtering can improve debugger performance dramatically by reducing
        /// the amount of event traffic sent from the target VM to the
        /// debugger.
        modifiers: &'a [Modifier<'a>],
    }

    /// Clear an event request.
    ///
    /// See [EventKind] for a complete list of events that can be cleared.
    ///
    /// Only the event request matching the specified event kind and
    /// `request_id` is cleared.
    ///
    /// If there isn't a matching event request the command is a no-op and does
    /// not result in an error.
    ///
    /// Automatically generated events do not have a corresponding event request
    /// and may not be cleared using this command.
    #[jdwp_command((), 15, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct Clear {
        /// Event kind to clear
        event_kind: EventKind,
        /// ID of request to clear
        request_id: RequestID,
    }

    /// Removes all set breakpoints, a no-op if there are no breakpoints set.
    #[jdwp_command((), 15, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ClearAllBreakpoints;
}

/// StackFrame Command Set (16)
pub mod stack_frame {
    use super::*;

    /// Returns the value of one or more local variables in a given frame.
    ///
    /// Each variable must be visible at the frame's code index.
    ///
    /// Even if local variable information is not available, values can be
    /// retrieved if the front-end is able to determine the correct local
    /// variable index. (Typically, this index can be determined for method
    /// arguments from the method signature without access to the local
    /// variable table information.)
    #[jdwp_command(C::Map<Value>, 16, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct GetValues<C: Coll<Item = (u32, Tag)>> {
        /// The frame's thread.
        pub thread_id: ThreadID,
        /// The frame ID.
        pub frame_id: FrameID,
        /// Local variable indices and types to get.
        pub slots: C,
    }

    /// Sets the value of one or more local variables.
    ///
    /// Each variable must be visible at the current frame code index. For
    /// primitive values, the value's type must match the variable's type
    /// exactly. For object values, there must be a widening reference
    /// conversion from the value's type to thevariable's type and the
    /// variable's type must be loaded.
    ///
    /// Even if local variable information is not available, values can be set,
    /// if the front-end is able to determine the correct local variable
    /// index. (Typically, thisindex can be determined for method arguments
    /// from the method signature without access to the local variable table
    /// information.)
    #[jdwp_command((), 16, 2)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct SetValues<'a> {
        /// The frame's thread.
        pub thread_id: ThreadID,
        /// The frame ID.
        pub frame_id: FrameID,
        /// Local variable indices and values to set.
        pub slots: &'a [(u32, Value)],
    }

    /// Returns the value of the 'this' reference for this frame.
    ///
    /// If the frame's method is static or native, the reply will contain the
    /// null object reference.
    #[jdwp_command(Option<TaggedObjectID>, 16, 3)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ThisObject {
        /// The frame's thread.
        pub thread_id: ThreadID,
        /// The frame ID.
        pub frame_id: FrameID,
    }

    /// Pop the top-most stack frames of the thread stack, up to, and including
    /// 'frame'. The thread must be suspended to perform this command. The
    /// top-most stack frames are discarded and the stack frame previous to
    /// 'frame' becomes the current frame. The operand stack is restored --
    /// the argument values are added back and if the invoke was not
    /// invokestatic, objectref is added back as well. The Java virtual
    /// machine program counter is restored to the opcode of the invoke
    /// instruction.
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
}

/// ClassObjectReference Command Set (17)
pub mod class_object_reference {
    use super::*;

    /// Returns the reference type reflected by this class object.
    #[jdwp_command(TaggedReferenceTypeID, 17, 1)]
    #[derive(Debug, Clone, JdwpWritable)]
    pub struct ReflectedType {
        /// The class object
        class_object_id: ClassObjectID,
    }
}

/// Event Command Set (64)
pub mod event {
    use super::*;

    pub(crate) mod sealed {
        /// This trait is used to reduce duplication between the JDWP-level and
        /// high-level event enums. The enum is parameterized by a
        /// domain, and there are two domains that are used:
        /// - [Spec] is used for the JDWP-level enum
        pub trait Domain {
            type Thread;
            type TaggedObject;
            type TaggedReferenceType;
            type Field;
        }
    }

    #[derive(Debug, Clone)]
    pub struct Spec;

    impl sealed::Domain for Spec {
        type Thread = ThreadID;
        type TaggedObject = TaggedObjectID;
        type TaggedReferenceType = TaggedReferenceTypeID;

        type Field = (TaggedReferenceTypeID, FieldID, Option<TaggedObjectID>);
    }

    /// The field order in JDWP spec is the discriminator (EventKind) and then
    /// the request id, that's why it's not moved out of the enum.
    #[derive(Debug, Clone, JdwpReadable)]
    #[repr(u8)]
    pub enum Event<D: sealed::Domain> {
        /// Notification of step completion in the target VM.
        ///
        /// The step event is generated before the code at its location is
        /// executed.
        SingleStep(
            /// Request that generated the event
            RequestID,
            /// Stepped thread
            D::Thread,
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
            /// Stepped thread
            D::Thread,
            /// Location stepped to
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
        /// In some VMs method entry events can occur for a particular thread
        /// before its thread start event occurs if methods are called
        /// as part of the thread's initialization.
        MethodEntry(
            /// Request that generated the event
            RequestID,
            /// Stepped thread
            D::Thread,
            /// Location stepped to
            Location,
        ) = EventKind::MethodEntry as u8,

        /// Notification of a method return in the target VM.
        ///
        /// This event is generated after all code in the method has executed,
        ///
        /// but the location of this event is the last executed location
        /// in the method.
        ///
        /// Method exit events are generated for both native and non-native
        /// methods.
        ///
        /// Method exit events are not generated if the method terminates with a
        /// thrown exception.
        MethodExit(
            /// Request that generated the event
            RequestID,
            /// Stepped thread
            D::Thread,
            /// Location stepped to
            Location,
        ) = EventKind::MethodExit as u8,

        /// Notification of a method return in the target VM.
        ///
        /// This event is generated after all code in the method has executed,
        ///
        /// but the location of this event is the last executed location
        /// in the method.
        ///
        /// Method exit events are generated for both native and non-native
        /// methods.
        ///
        /// Method exit events are not generated if the method terminates with a
        /// thrown exception.
        ///
        /// Since JDWP version 1.6.
        MethodExitWithReturnValue(
            /// Request that generated the event
            RequestID,
            /// Thread which exited method
            D::Thread,
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
            D::Thread,
            /// Monitor object reference
            D::TaggedObject,
            /// Location of contended monitor enter
            Location,
        ) = EventKind::MonitorContendedEnter as u8,

        /// Notification of a thread in the target VM is entering a monitor
        /// after waiting for it to be released by another thread.
        ///
        /// Requires `can_request_monitor_events` capability - see
        /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
        ///
        /// Since JDWP version 1.6.
        MonitorContendedEntered(
            /// Request that generated the event
            RequestID,
            /// Thread which entered monitor
            D::Thread,
            /// Monitor object reference
            D::TaggedObject,
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
            D::Thread,
            /// Monitor object reference
            D::TaggedObject,
            /// Location at which the wait will occur
            Location,
            /// Thread wait time in milliseconds
            u64,
        ) = EventKind::MonitorWait as u8,

        /// Notification that a thread in the target VM has finished waiting on
        /// a monitor object.
        ///
        /// Requires `can_request_monitor_events` capability - see
        /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
        ///
        /// Since JDWP version 1.6.
        MonitorWaited(
            /// Request that generated the event
            RequestID,
            /// Thread which waited
            D::Thread,
            /// Monitor object reference
            D::TaggedObject,
            /// Location at which the wait occurred
            Location,
            /// True if timed out
            bool,
        ) = EventKind::MonitorWaited as u8,

        /// Notification of an exception in the target VM.
        ///
        /// If the exception is thrown from a non-native method, the exception
        /// event is generated at the location where the exception is
        /// thrown.
        ///
        /// If the exception is thrown from a native method, the exception event
        /// is generated at the first non-native location reached after
        /// the exception is thrown.
        Exception(
            /// Request that generated the event
            RequestID,
            /// Thread with exception
            D::Thread,
            /// Location of exception throw (or first non-native location after
            /// throw if thrown from a native method)
            Location,
            /// Thrown exception
            D::TaggedObject,
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
            /// In such cases, it is not possible to predict whether an
            /// exception will be handled by some native method on
            /// the call stack.
            ///
            /// Thus, it is possible that exceptions considered uncaught here
            /// will, in fact, be handled by a native method and not
            /// cause termination of the target VM.
            ///
            /// Furthermore, it cannot be assumed that the catch location
            /// returned here will ever be reached by the throwing
            /// thread. If there is a native frame between the
            /// current location and the catch location, the
            /// exception might be handled and cleared in that
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
        /// The new thread can be the result of a call to
        /// `java.lang.Thread.start` or the result of attaching a new
        /// thread to the VM though JNI.
        ///
        /// The notification is generated by the new thread some time before its
        /// execution starts.
        ///
        /// Because of this timing, it is possible to receive other events for
        /// the thread before this event is received.
        ///
        /// (Notably, Method Entry Events and Method Exit Events might occur
        /// during thread initialization.
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
            D::Thread,
        ) = EventKind::ThreadStart as u8,

        /// Notification of a completed thread in the target VM.
        ///
        /// The notification is generated by the dying thread before it
        /// terminates.
        ///
        /// Because of this timing, it is possible for
        /// [AllThreads](super::virtual_machine::AllThreads) to return this
        /// thread after this event is received.
        ///
        /// Note that this event gives no information about the lifetime of the
        /// thread object.
        ///
        /// It may or may not be collected soon depending on what references
        /// exist in the target VM.
        ThreadDeath(
            /// Request that generated the event
            RequestID,
            /// Ending thread
            D::Thread,
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
            /// Debugger threads take precautions to prevent these events, but
            /// they cannot be avoided under some conditions,
            /// especially for some subclasses of `java.lang.Error`.
            ///
            /// If the event was generated by a debugger system thread, the
            /// value returned by this method is null, and if the
            /// requested suspend policy for the event was
            /// [EventThread](SuspendPolicy::EventThread)
            /// all threads will be suspended instead, and the composite event's
            /// suspend policy will reflect this change.
            ///
            /// Note that the discussion above does not apply to system threads
            /// created by the target VM during its normal (non-debug)
            /// operation.
            D::Thread,
            /// Type being prepared
            D::TaggedReferenceType,
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
            D::Thread,
            /// Location of access
            Location,
            /// Field being accessed
            D::Field,
        ) = EventKind::FieldAccess as u8,

        /// Notification of a field modification in the target VM. Requires
        /// `can_watch_field_modification` capability - see
        /// [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
        FieldModification(
            /// Request that generated the event
            RequestID,
            /// Modifying thread
            D::Thread,
            /// Location of modify
            Location,
            /// Field being modified
            D::Field,
            /// Value to be assigned
            Value,
        ) = EventKind::FieldModification as u8,

        /// Notification of initialization of a target VM.
        ///
        /// This event is received before the main thread is started and before
        /// any application code has been executed.
        ///
        /// Before this event occurs a significant amount of system code has
        /// executed and a number of system classes have been loaded.
        ///
        /// This event is always generated by the target VM, even if not
        /// explicitly requested.
        VmStart(
            /// Request that generated the event (or None if this event is
            /// automatically generated)
            Option<RequestID>,
            /// Initial thread
            D::Thread,
        ) = EventKind::VmStart as u8,

        VmDeath(
            /// Request that generated the event
            Option<RequestID>,
        ) = EventKind::VmDeath as u8,
    }

    impl<D: sealed::Domain> Event<D> {
        pub fn kind(&self) -> EventKind {
            // SAFETY: Self and EventKind fulfill the requirements
            unsafe { crate::spec::tag(self) }
        }
    }

    /// Several events may occur at a given time in the target VM. For example,
    /// there may be more than one breakpoint request for a given location or
    /// you might single step to the same location as a breakpoint request.
    /// These events are delivered together as a composite event. For
    /// uniformity, a composite event is always used to deliver events, even if
    /// there is only one event to report.
    ///
    /// The events that are grouped in a composite event are restricted in the
    /// following ways:
    /// - Only with other thread start events for the same thread:
    ///     - Thread Start Event
    /// - Only with other thread death events for the same thread:
    ///     - Thread Death Event
    /// - Only with other class prepare events for the same class:
    ///     - Class Prepare Event
    /// - Only with other class unload events for the same class:
    ///     - Class Unload Event
    /// - Only with other access watchpoint events for the same field access:
    ///     - Access Watchpoint Event
    /// - Only with other modification watchpoint events for the same field
    ///   modification:
    ///     - Modification Watchpoint Event
    /// - Only with other Monitor contended enter events for the same monitor
    ///   object:
    ///     - Monitor Contended Enter Event
    /// - Only with other Monitor contended entered events for the same monitor
    ///   object:
    ///     - Monitor Contended Entered Event
    /// - Only with other Monitor wait events for the same monitor object:
    ///     - Monitor Wait Event
    /// - Only with other Monitor waited events for the same monitor object:
    ///     - Monitor Waited Event
    /// - Only with other ExceptionEvents for the same exception occurrance:
    ///     - ExceptionEvent
    /// - Only with other members of this group, at the same location and in the
    ///   same thread:
    ///     - Breakpoint Event
    ///     - Step Event
    ///     - Method Entry Event
    ///     - Method Exit Event
    ///
    /// The VM Start Event and VM Death Event are automatically generated
    /// events. This means they do not need to be requested using the
    /// [event_request::Set] command. The VM Start event signals the completion
    /// of VM initialization. The VM Death event signals the termination of
    /// the VM.If there is a debugger connected at the time when an
    /// automatically generated event occurs it is sent from the target VM.
    /// Automatically generated events may also be requested using the
    /// [event_request::Set] command and thus multiple events of the same
    /// event kind will be sent from the target VM when an event
    /// occurs. Automatically generated events are sent with the `request_id`
    /// field in the Event Data set to None. The value of the `suspend_policy`
    /// field in the Event Data depends on the event. For the automatically
    /// generated VM Start Event the value of `suspend_policy` is not defined
    /// and is therefore implementation or configuration specific.
    /// In the Sun implementation, for example, the `suspend_policy` is
    /// specified as an option to the JDWP agent at launch-time.The
    /// automatically generated VM Death Event will have the
    /// `suspend_policy` set to [None](SuspendPolicy::None).
    #[jdwp_command((), 64, 100)]
    #[derive(Debug, Clone, JdwpReadable)]
    pub struct Composite {
        /// Which threads where suspended by this composite event?
        pub suspend_policy: SuspendPolicy,
        /// Events in set.
        pub events: Vec<Event<Spec>>,
    }
}
