use std::{
    fmt::{Display, Formatter},
    io::{self, Error, ErrorKind, Read, Write},
};

use bitflags::bitflags;

use crate::codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter};

macro_rules! readable_enum {
    ($e:ident: $repr:ident, $($name:ident = $id:literal | $string:literal),* $(,)?) => {
        #[repr($repr)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $e {
            $(
                #[doc = $string]
                $name = $id,
            )*
        }

        impl $e {
            pub fn from(n: $repr) -> Option<Self> {
                match n {
                    $($id => Some($e::$name),)*
                    _ => None
                }
            }
        }

        impl JdwpReadable for $e {
            fn read<R: Read>(read: &mut JdwpReader<R>) -> std::io::Result<Self> {
                match $repr::read(read)? {
                    $($id => Ok($e::$name),)*
                    _ => Err(Error::from(ErrorKind::InvalidData))
                }
            }
        }

        impl JdwpWritable for $e {
            fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> std::io::Result<()> {
                (*self as $repr).write(write)
            }
        }
    };
    ($e:ident: $repr:ident, $($name:ident = $id:literal),* $(,)?) => {
        readable_enum!($e: $repr, $($name = $id | "",)*);
    };
    ($e:ident: $repr:ident | Display, $($name:ident = $id:literal | $string:literal),* $(,)?) => {
        readable_enum!($e: $repr, $($name = $id | $string,)*);

        impl Display for $e {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    $($e::$name => $string,)*
                })
            }
        }
    };
}

readable_enum! {
    ErrorCode: u16 | Display,

    None = 0 | "No error has occurred",
    InvalidThread = 10 | "Passed thread is null, is not a valid thread or has exited",
    InvalidThreadGroup = 11 | "Thread group invalid",
    InvalidPriority = 12 | "Invalid priority",
    ThreadNotSuspended = 13 | "If the specified thread has not been suspended by an event",
    ThreadSuspended = 14 | "Thread already suspended",
    ThreadNotAlive = 15 | "Thread has not been started or is now dead",
    InvalidObject = 20 | "If this reference type has been unloaded and garbage collected",
    InvalidClass = 21 | "Invalid class",
    ClassNotPrepared = 22 | "Class has been loaded but not yet prepared",
    InvalidMethodid = 23 | "Invalid method",
    InvalidLocation = 24 | "Invalid location",
    InvalidFieldid = 25 | "Invalid field",
    InvalidFrameid = 30 | "Invalid jframeID",
    NoMoreFrames = 31 | "There are no more Java or JNI frames on the call stack",
    OpaqueFrame = 32 | "Information about the frame is not available",
    NotCurrentFrame = 33 | "Operation can only be performed on current frame",
    TypeMismatch = 34 | "The variable is not an appropriate type for the function used",
    InvalidSlot = 35 | "Invalid slot",
    Duplicate = 40 | "Item already set",
    NotFound = 41 | "Desired element not found",
    InvalidMonitor = 50 | "Invalid monitor",
    NotMonitorOwner = 51 | "This thread doesn't own the monitor",
    Interrupt = 52 | "The call has been interrupted before completion",
    InvalidClassFormat = 60 | "The virtual machine attempted to read a class file and determined that the file is malformed or otherwise cannot be interpreted as a class file",
    CircularClassDefinition = 61 | "A circularity has been detected while initializing a class",
    FailsVerification = 62 | "The verifier detected that a class file, though well formed, contained some sort of internal inconsistency or security problem",
    AddMethodNotImplemented = 63 | "Adding methods has not been implemented",
    SchemaChangeNotImplemented = 64 | "Schema change has not been implemented",
    InvalidTypestate = 65 | "The state of the thread has been modified, and is now inconsistent",
    HierarchyChangeNotImplemented = 66 | "A direct superclass is different for the new class version, or the set of directly implemented interfaces is different and canUnrestrictedlyRedefineClasses is false",
    DeleteMethodNotImplemented = 67 | "The new class version does not declare a method declared in the old class version and canUnrestrictedlyRedefineClasses is false",
    UnsupportedVersion = 68 | "A class file has a version number not supported by this VM",
    NamesDontMatch = 69 | "The class name defined in the new class file is different from the name in the old class object",
    ClassModifiersChangeNotImplemented = 70 | "The new class version has different modifiers and and canUnrestrictedlyRedefineClasses is false",
    MethodModifiersChangeNotImplemented = 71 | "A method in the new class version has different modifiers than its counterpart in the old class version and and canUnrestrictedlyRedefineClasses is false",
    NotImplemented = 99 | "The functionality is not implemented in this virtual machine",
    NullPointer = 100 | "Invalid pointer",
    AbsentInformation = 101 | "Desired information is not available",
    InvalidEventType = 102 | "The specified event type id is not recognized",
    IllegalArgument = 103 | "Illegal argument",
    OutOfMemory = 110 | "The function needed to allocate memory and no more memory was available for allocation",
    AccessDenied = 111 | "Debugging has not been enabled in this virtual machine. JVMTI cannot be used",
    VmDead = 112 | "The virtual machine is not running",
    Internal = 113 | "An unexpected internal error has occurred",
    UnattachedThread = 115 | "The thread being used to call this function is not attached to the virtual machine. Calls must be made from attached threads",
    InvalidTag = 500 | "object type id or class tag",
    AlreadyInvoking = 502 | "Previous invoke not complete",
    InvalidIndex = 503 | "Index is invalid",
    InvalidLength = 504 | "The length is invalid",
    InvalidString = 506 | "The string is invalid",
    InvalidClassLoader = 507 | "The class loader is invalid",
    InvalidArray = 508 | "The array is invalid",
    TransportLoad = 509 | "Unable to load the transport",
    TransportInit = 510 | "Unable to initialize the transport",
    NativeMethod = 511 | "NATIVE_METHOD",
    InvalidCount = 512 | "The count is invalid",
}

readable_enum! {
    EventKind: u8,

    SingleStep = 1,
    Breakpoint = 2,
    FramePop = 3,
    Exception = 4,
    UserDefined = 5,
    ThreadStart = 6,
    ThreadDeath = 7,
    ClassPrepare = 8,
    ClassUnload = 9,
    ClassLoad = 10,
    FieldAccess = 20,
    FieldModification = 21,
    ExceptionCatch = 30,
    MethodEntry = 40,
    MethodExit = 41,
    MethodExitWithReturnValue = 42,
    MonitorContendedEnter = 43,
    MonitorContendedEntered = 44,
    MonitorWait = 45,
    MonitorWaited = 46,
    VmStart = 90,
    VmDeath = 99,
    VmDisconnected = 100,
}

readable_enum! {
    ThreadStatus: u32,

    Zombie = 0,
    Running	= 1,
    Sleeping = 2,
    Monitor	= 3,
    Wait = 4,
}

readable_enum! {
    SuspendStatus: u32,

    NotSuspended = 0,
    Suspended = 1,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ClassStatus: u32 {
        const VERIFIED = 1;
        const PREPARED = 2;
        const INITIALIZED = 4;
        const ERROR = 8;

        const OK = Self::VERIFIED.bits() | Self::PREPARED.bits() | Self::INITIALIZED.bits();
    }
}

impl JdwpReadable for ClassStatus {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        Self::from_bits(u32::read(read)?).ok_or_else(|| Error::from(ErrorKind::InvalidData))
    }
}

impl JdwpWritable for ClassStatus {
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        self.bits().write(write)
    }
}

readable_enum! {
    TypeTag: u8,

    Class = 1 | "ReferenceType is a class",
    Interface = 2 | "ReferenceType is an interface",
    Array = 3 | "ReferenceType is an array",
}

readable_enum! {
    Tag: u8,

    Array = 91 | "'[' - an array object ([ObjectID](crate::types::ObjectID) size).",
    Byte = 66 | "'B' - a byte value (1 byte).",
    Char = 67 | "'C' - a character value (2 bytes).",
    Object = 76 | "'L' - an object ([ObjectID](crate::types::ObjectID) size).",
    Float = 70 | "'F' - a float value (4 bytes).",
    Double = 68 | "'D' - a double value (8 bytes).",
    Int = 73 | "'I' - an int value (4 bytes).",
    Long = 74 | "'J' - a long value (8 bytes).",
    Short = 83 | "'S' - a short value (2 bytes).",
    Void = 86 | "'V' - a void value (no bytes).",
    Boolean = 90 | "'Z' - a boolean value (1 byte).",
    String = 115 | "'s' - a String object ([ObjectID](crate::types::ObjectID) size).",
    Thread = 116 | "'t' - a Thread object ([ObjectID](crate::types::ObjectID) size).",
    ThreadGroup = 103 | "'g' - a ThreadGroup object ([ObjectID](crate::types::ObjectID) size).",
    ClassLoader = 108 | "'l' - a ClassLoader object ([ObjectID](crate::types::ObjectID) size).",
    ClassObject = 99 | "'c' - a class object object ([ObjectID](crate::types::ObjectID) size).",
}

readable_enum! {
    StepDepth: u32,

    Into = 0 | "Step into any method calls that occur before the end of the step",
    Over = 1 | "Step over any method calls that occur before the end of the step",
    Out = 2 | "Step out of the current method",
}

readable_enum! {
    StepSize: u32,

    Min = 0 | "Step by the minimum possible amount (often a byte code instruction)",
    Line = 1 | "Step to the next source line unless there is no line number information in which case a MIN step is done instead",
}

readable_enum! {
    SuspendPolicy: u8,

    None = 0 | "Suspend no threads when this event is encountered",
    EventThread = 1 | "Suspend the event thread when this event is encountered",
    All = 2 | "Suspend all threads when this event is encountered",
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct InvokeOptions: u32 {
        /// otherwise, all threads started
        const SINGLE_THREADED = 0x01;
        /// otherwise, normal virtual invoke (instance methods only)
        const NONVIRTUAL = 0x02;
    }
}

impl JdwpReadable for InvokeOptions {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        Self::from_bits(u32::read(read)?).ok_or_else(|| Error::from(ErrorKind::InvalidData))
    }
}

impl JdwpWritable for InvokeOptions {
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        self.bits().write(write)
    }
}

readable_enum! {
    ModifierKind: u8,

    Count = 1 | "Limit the requested event to be reported at most once after a given number of occurrences. The event is not reported the first count - 1 times this filter is reached. To request a one-off event, call this method with a count of 1.",
    Conditional = 2 | "Conditional on expression",
    ThreadOnly = 3 | "Restricts reported events to those in the given thread. This modifier can be used with any event kind except for class unload.",
    ClassOnly = 4 | "For class prepare events, restricts the events generated by this request to be the preparation of the given reference type and any subtypes. For monitor wait and waited events, restricts the events generated by this request to those whose monitor object is of the given reference type or any of its subtypes. For other events, restricts the events generated by this request to those whose location is in the given reference type or any of its subtypes. An event will be generated for any location in a reference type that can be safely cast to the given reference type. This modifier can be used with any event kind except class unload, thread start, and thread end.",
    ClassMatch = 5 | "Restricts reported events to those for classes whose name matches the given restricted regular expression. For class prepare events, the prepared class name is matched. For class unload events, the unloaded class name is matched. For monitor wait and waited events, the name of the class of the monitor object is matched. For other events, the class name of the event's location is matched. This modifier can be used with any event kind except thread start and thread end.",
    ClassExclude = 6 | "Restricts reported events to those for classes whose name does not match the given restricted regular expression. For class prepare events, the prepared class name is matched. For class unload events, the unloaded class name is matched. For monitor wait and waited events, the name of the class of the monitor object is matched. For other events, the class name of the event's location is matched. This modifier can be used with any event kind except thread start and thread end.",
    LocationOnly = 7 | "Restricts reported events to those that occur at the given location. This modifier can be used with breakpoint, field access, field modification, step, and exception event kinds.",
    ExceptionOnly = 8 | "Restricts reported exceptions by their class and whether they are caught or uncaught. This modifier can be used with exception event kinds only.",
    FieldOnly = 9 | "Restricts reported events to those that occur for a given field. This modifier can be used with field access and field modification event kinds only.",
    Step = 10 | "Restricts reported step events to those which satisfy depth and size constraints. This modifier can be used with step event kinds only.",
    InstanceOnly = 11 | "Restricts reported events to those whose active 'this' object is the given object. Match value is the null object for static methods. This modifier can be used with any event kind except class prepare, class unload, thread start, and thread end. Introduced in JDWP version 1.4.",
    SourceNameMatch = 12 | "",
}
