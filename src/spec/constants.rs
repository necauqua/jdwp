use std::{
    fmt::{Display, Formatter},
    io::{self, Error, ErrorKind, Read, Write},
};

use bitflags::bitflags;

use crate::codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter};

macro_rules! jdwp_enum {
    (
        #[repr($repr:ident)]
        pub enum $e:ident {
            $($(#[doc = $string:literal])* $name:ident = $id:literal),*
            $(,)?
        }
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[repr($repr)]
        pub enum $e {
            $($(#[doc = $string])* $name = $id,)*
        }

        impl TryFrom<$repr> for $e {
            type Error = $repr;

            fn try_from(value: $repr) -> Result<Self, Self::Error> {
                match value {
                    $($id => Ok($e::$name),)*
                    other => Err(other),
                }
            }
        }

        impl JdwpReadable for $e {
            fn read<R: Read>(read: &mut JdwpReader<R>) -> std::io::Result<Self> {
                Self::try_from($repr::read(read)?)
                    .map_err(|_| Error::from(ErrorKind::InvalidData))
            }
        }

        impl JdwpWritable for $e {
            fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> std::io::Result<()> {
                (*self as $repr).write(write)
            }
        }
    };
    (
        #[derive(Display)]
        #[repr($repr:ident)]
        pub enum $e:ident {
            $(#[doc = $string:literal] $name:ident = $id:literal),*
            $(,)?
        }
    ) => {
        jdwp_enum! {
            #[repr($repr)]
            pub enum $e {
                $(#[doc = $string] $name = $id,)*
            }
        }

        impl Display for $e {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    $($e::$name => $string,)*
                })
            }
        }
    };
    (
        $(
            $(#[derive(Display)])?
            #[repr($repr:ident)]
            pub enum $e:ident {
                $(#[doc = $string:literal] $name:ident = $id:literal),*
                $(,)?
            }
        )*
    ) => {
        $(
            jdwp_enum! {
                $(#[derive(Display)])?
                #[repr($repr)]
                pub enum $e {
                    $(#[doc = $string] $name = $id,)*
                }
            }
        )*
    };
}

jdwp_enum! {
    #[derive(Display)]
    #[repr(u16)]
    pub enum ErrorCode {
        /// No error has occurred
        None = 0,
        /// Passed thread is null, is not a valid thread or has exited
        InvalidThread = 10,
        /// Thread group invalid
        InvalidThreadGroup = 11,
        /// Invalid priority
        InvalidPriority = 12,
        /// If the specified thread has not been suspended by an event
        ThreadNotSuspended = 13,
        /// Thread already suspended
        ThreadSuspended = 14,
        /// Thread has not been started or is now dead
        ThreadNotAlive = 15,
        /// If this reference type has been unloaded and garbage collected
        InvalidObject = 20,
        /// Invalid class
        InvalidClass = 21,
        /// Class has been loaded but not yet prepared
        ClassNotPrepared = 22,
        /// Invalid method
        InvalidMethodid = 23,
        /// Invalid location
        InvalidLocation = 24,
        /// Invalid field
        InvalidFieldid = 25,
        /// Invalid jframeID
        InvalidFrameid = 30,
        /// There are no more Java or JNI frames on the call stack
        NoMoreFrames = 31,
        /// Information about the frame is not available
        OpaqueFrame = 32,
        /// Operation can only be performed on current frame
        NotCurrentFrame = 33,
        /// The variable is not an appropriate type for the function used
        TypeMismatch = 34,
        /// Invalid slot
        InvalidSlot = 35,
        /// Item already set
        Duplicate = 40,
        /// Desired element not found
        NotFound = 41,
        /// Invalid monitor
        InvalidMonitor = 50,
        /// This thread doesn't own the monitor
        NotMonitorOwner = 51,
        /// The call has been interrupted before completion
        Interrupt = 52,
        /// The virtual machine attempted to read a class file and determined that the file is malformed or otherwise cannot be interpreted as a class file
        InvalidClassFormat = 60,
        /// A circularity has been detected while initializing a class
        CircularClassDefinition = 61,
        /// The verifier detected that a class file, though well formed, contained some sort of internal inconsistency or security problem
        FailsVerification = 62,
        /// Adding methods has not been implemented
        AddMethodNotImplemented = 63,
        /// Schema change has not been implemented
        SchemaChangeNotImplemented = 64,
        /// The state of the thread has been modified, and is now inconsistent
        InvalidTypestate = 65,
        /// A direct superclass is different for the new class version, or the set of directly implemented interfaces is different and canUnrestrictedlyRedefineClasses is false
        HierarchyChangeNotImplemented = 66,
        /// The new class version does not declare a method declared in the old class version and canUnrestrictedlyRedefineClasses is false
        DeleteMethodNotImplemented = 67,
        /// A class file has a version number not supported by this VM
        UnsupportedVersion = 68,
        /// The class name defined in the new class file is different from the name in the old class object
        NamesDontMatch = 69,
        /// The new class version has different modifiers and and canUnrestrictedlyRedefineClasses is false
        ClassModifiersChangeNotImplemented = 70,
        /// A method in the new class version has different modifiers than its counterpart in the old class version and and canUnrestrictedlyRedefineClasses is false
        MethodModifiersChangeNotImplemented = 71,
        /// The functionality is not implemented in this virtual machine
        NotImplemented = 99,
        /// Invalid pointer
        NullPointer = 100,
        /// Desired information is not available
        AbsentInformation = 101,
        /// The specified event type id is not recognized
        InvalidEventType = 102,
        /// Illegal argument
        IllegalArgument = 103,
        /// The function needed to allocate memory and no more memory was available for allocation
        OutOfMemory = 110,
        /// Debugging has not been enabled in this virtual machine. JVMTI cannot be used
        AccessDenied = 111,
        /// The virtual machine is not running
        VmDead = 112,
        /// An unexpected internal error has occurred
        Internal = 113,
        /// The thread being used to call this function is not attached to the virtual machine. Calls must be made from attached threads
        UnattachedThread = 115,
        /// object type id or class tag
        InvalidTag = 500,
        /// Previous invoke not complete
        AlreadyInvoking = 502,
        /// Index is invalid
        InvalidIndex = 503,
        /// The length is invalid
        InvalidLength = 504,
        /// The string is invalid
        InvalidString = 506,
        /// The class loader is invalid
        InvalidClassLoader = 507,
        /// The array is invalid
        InvalidArray = 508,
        /// Unable to load the transport
        TransportLoad = 509,
        /// Unable to initialize the transport
        TransportInit = 510,
        /// NATIVE_METHOD
        NativeMethod = 511,
        /// The count is invalid
        InvalidCount = 512,
    }
}

jdwp_enum! {
    #[repr(u8)]
    pub enum EventKind {
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
        /// Never sent across JDWP
        VmDisconnected = 100,
    }
}

jdwp_enum! {
    #[repr(u32)]
    pub enum ThreadStatus {
        Zombie = 0,
        Running = 1,
        Sleeping = 2,
        Monitor = 3,
        Wait = 4,
    }
}

jdwp_enum! {
    #[repr(u32)]
    pub enum SuspendStatus {
        NotSuspended = 0,
        Suspended = 1,
    }
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

pub(crate) trait ByteTag {}

jdwp_enum! {
    #[repr(u8)]
    pub enum TypeTag {
        /// ReferenceType is a class
        Class = 1,
        /// ReferenceType is an interface
        Interface = 2,
        /// ReferenceType is an array
        Array = 3,
    }
}

jdwp_enum! {
    #[repr(u8)]
    pub enum Tag {
        /// '[' - an array object ([ObjectID](crate::spec::ObjectID) size).
        Array = 91,
        /// 'B' - a byte value (1 byte).
        Byte = 66,
        /// 'C' - a character value (2 bytes).
        Char = 67,
        /// 'L' - an object ([ObjectID](crate::spec::ObjectID) size).
        Object = 76,
        /// 'F' - a float value (4 bytes).
        Float = 70,
        /// 'D' - a double value (8 bytes).
        Double = 68,
        /// 'I' - an int value (4 bytes).
        Int = 73,
        /// 'J' - a long value (8 bytes).
        Long = 74,
        /// 'S' - a short value (2 bytes).
        Short = 83,
        /// 'V' - a void value (no bytes).
        Void = 86,
        /// 'Z' - a boolean value (1 byte).
        Boolean = 90,
        /// 's' - a String object ([ObjectID](crate::spec::ObjectID) size).
        String = 115,
        /// 't' - a Thread object ([ObjectID](crate::spec::ObjectID) size).
        Thread = 116,
        /// 'g' - a ThreadGroup object ([ObjectID](crate::spec::ObjectID) size).
        ThreadGroup = 103,
        /// 'l' - a ClassLoader object ([ObjectID](crate::spec::ObjectID) size).
        ClassLoader = 108,
        /// 'c' - a class object object ([ObjectID](crate::spec::ObjectID) size).
        ClassObject = 99,
    }
}

impl ByteTag for Tag {}
impl ByteTag for TypeTag {}
impl ByteTag for EventKind {}

impl<T> JdwpReadable for Option<T>
where
    T: ByteTag + TryFrom<u8>,
{
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        Ok(match u8::read(read)? {
            0 => None,
            raw => Some(T::try_from(raw).map_err(|_| Error::from(ErrorKind::InvalidData))?),
        })
    }
}

jdwp_enum! {
    #[repr(u32)]
    pub enum StepDepth {
        /// Step into any method calls that occur before the end of the step
        Into = 0,
        /// Step over any method calls that occur before the end of the step
        Over = 1,
        /// Step out of the current method
        Out = 2,
    }
}

jdwp_enum! {
    #[repr(u32)]
    pub enum StepSize {
        /// Step by the minimum possible amount (often a byte code instruction)
        Min = 0,
        /// Step to the next source line unless there is no line number
        /// information in which case a MIN step is done instead
        Line = 1,
    }
}

jdwp_enum! {
    #[repr(u8)]
    pub enum SuspendPolicy {
        /// Suspend no threads when this event is encountered
        None = 0,
        /// Suspend the event thread when this event is encountered
        EventThread = 1,
        /// Suspend all threads when this event is encountered
        All = 2,
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct InvokeOptions: u32 {
        const NONE = 0x00;
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

jdwp_enum! {
    #[repr(u8)]
    pub enum ModifierKind {
        /// Limit the requested event to be reported at most once after a given
        /// number of occurrences.
        ///
        /// The event is not reported the first count - 1 times this filter is
        /// reached. To request a one-off event, call this method with a count
        /// of 1.
        Count = 1,
        /// Conditional on expression
        Conditional = 2,
        /// Restricts reported events to those in the given thread.
        /// This modifier can be used with any event kind except for class
        /// unload.
        ThreadOnly = 3,
        /// For class prepare events, restricts the events generated by this
        /// request to be the preparation of the given reference type and any
        /// subtypes.
        ///
        /// For monitor wait and waited events, restricts the events generated
        /// by this request to those whose monitor object is of the given
        /// reference type or any of its subtypes.
        ///
        /// For other events, restricts the events generated by this request to
        /// those whose location is in the given reference type or any of its
        /// subtypes.
        ///
        /// An event will be generated for any location in a reference type
        /// that can be safely cast to the given reference type.
        ///
        /// This modifier can be used with any event kind except class unload,
        /// thread start, and thread end.
        ClassOnly = 4,
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
        /// For other events, the class name of the event's location is
        /// matched.
        ///
        /// This modifier can be used with any event kind except thread start
        /// and thread end.
        ClassMatch = 5,
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
        /// For other events, the class name of the event's location is
        /// matched.
        ///
        /// This modifier can be used with any event kind except thread start
        /// and thread end.
        ClassExclude = 6,
        /// Restricts reported events to those that occur at the given
        /// location.
        ///
        /// This modifier can be used with breakpoint, field access, field
        /// modification, step, and exception event kinds.
        LocationOnly = 7,
        /// Restricts reported exceptions by their class and whether they are
        /// caught or uncaught.
        ///
        /// This modifier can be used with exception event kinds only.
        ExceptionOnly = 8,
        /// Restricts reported events to those that occur for a given field.
        ///
        /// This modifier can be used with field access and field modification
        /// event kinds only.
        FieldOnly = 9,
        /// Restricts reported step events to those which satisfy depth and
        /// size constraints.
        ///
        /// This modifier can be used with step event kinds only.
        Step = 10,
        /// Restricts reported events to those whose active 'this' object is
        /// the given object. Match value is the null object for static
        /// methods.
        ///
        /// This modifier can be used with any event kind except class prepare,
        /// class unload, thread start, and thread end.
        ///
        /// Introduced in JDWP version 1.4.
        InstanceOnly = 11,

        // no jdwp doc, todo write something here I guess lol
        SourceNameMatch = 12,
    }
}
