use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    enums::{Tag, TypeTag},
};
use std::{
    fmt::{Debug, Formatter},
    io::{self, Read, Write},
    ops::Deref,
};

use crate::enums::{ModifierKind, StepDepth, StepSize};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

// why no #[doc = concat!(..)] in stable :(
macro_rules! docs {
    ($str:expr, $($x:tt)*) => {
        #[doc = $str]
        $($x)*
    };
}

/// Uniquely identifies an object in the target VM.
///
/// A particular object will be identified by exactly one [ObjectID] in JDWP
/// commands and replies throughout its lifetime (or until the [ObjectID] is
/// explicitly disposed). An [ObjectID] is not reused to identify a different
/// object unless it has been explicitly disposed, regardless of whether the
/// referenced object has been garbage collected.
///
/// Note that the existence of an object ID does not prevent the garbage
/// collection of the object.
/// Any attempt to access a a garbage collected object with its object ID will
/// result in the INVALID_OBJECT error code.
/// Garbage collection can be disabled with the [DisableCollection] command,
/// but it is not usually necessary to do so.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ObjectID(u64);

/// Uniquely identifies a method in some class in the target VM.
///
/// The [MethodID] must uniquely identify the method within its class/interface
/// or any of its subclasses/subinterfaces/implementors.
///
/// A [MethodID] is not necessarily unique on its own; it is always paired with a
/// [ReferenceTypeID] to uniquely identify one method.
///
/// The [ReferenceTypeID] can identify either the declaring type of the method or
/// a subtype.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MethodID(u64);

/// Uniquely identifies a field in some class in the target VM.
///
/// The [FieldID] must uniquely identify the field within its class/interface
/// or any of its subclasses/subinterfaces/implementors.
///
/// A [FieldID] is not necessarily unique on its own; it is always paired with
/// a [ReferenceTypeID] to uniquely identify one field.
///
/// The [ReferenceTypeID] can identify either the declaring type of the field
/// or a subtype.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FieldID(u64);

/// Uniquely identifies a frame in the target VM.
///
/// The [FrameID] must uniquely identify the frame within the entire VM (not
/// only within a given thread).
///
/// The [FrameID] need only be valid during the time its thread is suspended.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FrameID(u64);

/// Uniquely identifies a reference type in the target VM.
///
/// It should not be assumed that for a particular class, the [ClassObjectID]
/// and the [ReferenceTypeID] are the same.
///
/// A particular reference type will be identified by exactly one ID in JDWP
/// commands and replies throughout its lifetime A [ReferenceTypeID] is not
/// reused to identify a different reference type, regardless of whether the
/// referenced class has been unloaded.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ReferenceTypeID(u64);

macro_rules! ids {
    ($($id:ident: $tpe:ident),* $(,)?) => {
        $(
            impl $tpe {
                docs! {
                    concat!("Creates a new instance of [", stringify!($tpe), "] from an arbitrary number."),
                    /// # Safety
                    /// It is up to the caller to ensure that the id does indeed exist and is correct.
                    /// You can 'safely' obtain it by [JdwpReadable::read], but that will be incorrect obviously.
                    pub const unsafe fn new(raw: u64) -> Self {
                        Self(raw)
                    }
                }
            }

            impl Debug for $tpe {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, concat!(stringify!($tpe), "({})"), self.0)
                }
            }

            impl JdwpReadable for $tpe {
                fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
                    let id_size = read.id_sizes.$id as usize;
                    read.read_uint::<BigEndian>(id_size).map($tpe)
                }
            }

            impl JdwpWritable for $tpe {
                fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
                    let id_size = write.id_sizes.$id as usize;
                    write.write_uint::<BigEndian>(self.0, id_size)
                }
            }

            impl JdwpReadable for Option<$tpe> {
                fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
                    let id = $tpe::read(read)?;
                    Ok(if id.0 == 0 { None } else { Some(id) })
                }
            }

            impl JdwpWritable for Option<$tpe> {
                #[inline]
                fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
                    self.unwrap_or($tpe(0)).write(write)
                }
            }
        )*
    };
}

ids! {
    object_id_size: ObjectID,
    method_id_size: MethodID,
    field_id_size: FieldID,
    frame_id_size: FrameID,
    reference_type_id_size: ReferenceTypeID,
}

/// Uniquely identifies an object in the target VM that is known to be a thread.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ThreadID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a thread group.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ThreadGroupID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a string
/// object.
///
/// Note: this is very different from string, which is a value.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct StringID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a class
/// loader object.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ClassLoaderID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a class
/// object.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ClassObjectID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be an array.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ArrayID(ObjectID);

/// Uniquely identifies a reference type in the target VM that is known to be
/// a class type.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ClassID(ReferenceTypeID);

/// Uniquely identifies a reference type in the target VM that is known to be
/// an interface type.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct InterfaceID(ReferenceTypeID);

/// Uniquely identifies a reference type in the target VM that is known to be
/// an array type.
#[derive(Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ArrayTypeID(ReferenceTypeID);

macro_rules! wrapper_ids {
    ($($deref:ident {$($tpe:ident),* $(,)?})*) => {
        $($(
            impl $tpe {
                docs! {
                    concat!("Creates a new instance of [", stringify!($tpe), "] from an arbitrary [", stringify!($deref), "]."),
                    /// # Safety
                    /// It is up to the caller to ensure that the id does indeed correspond to this type.
                    /// You can 'safely' obtain it by [JdwpReadable::read], but that will be incorrect obviously.
                    pub const unsafe fn new(id: $deref) -> Self {
                        Self(id)
                    }
                }
            }

            impl Debug for $tpe {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, concat!(stringify!($tpe), "({})"), self.0.0)
                }
            }

            impl Deref for $tpe {
                type Target = $deref;

               #[inline]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl JdwpReadable for Option<$tpe> {
                fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
                    let id = $tpe::read(read)?;
                    Ok(if id.0 .0 == 0 { None } else { Some(id) })
                }
            }

            impl JdwpWritable for Option<$tpe> {
                fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
                    self.unwrap_or($tpe($deref(0))).write(write)
                }
            }
        )*)*
    };
}

wrapper_ids! {
    ObjectID {
        ThreadID,
        ThreadGroupID,
        StringID,
        ClassLoaderID,
        ClassObjectID,
        ArrayID,
    }
    ReferenceTypeID {
        ClassID,
        InterfaceID,
        ArrayTypeID,
    }
}

/// A value retrieved from the target VM.
/// This value can be an [ObjectID] or a primitive value (1 to 8 bytes).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    /// a void value (no bytes)
    Void,
    /// a byte value (1 byte)
    Byte(u8),
    /// a boolean value (1 byte)
    Boolean(bool),
    /// a character value (2 bytes)
    Char(u16),
    /// a short value (2 bytes)
    Short(i16),
    /// an int value (4 bytes)
    Int(i32),
    /// a long value (8 bytes)
    Long(i64),
    /// a float value (4 bytes)
    Float(f32),
    /// a double value (8 bytes)
    Double(f64),
    /// an object ([ObjectID] size)
    Object(ObjectID),
}

impl Value {
    pub fn tag(self) -> Tag {
        match self {
            Value::Void => Tag::Void,
            Value::Byte(_) => Tag::Byte,
            Value::Boolean(_) => Tag::Boolean,
            Value::Char(_) => Tag::Char,
            Value::Short(_) => Tag::Short,
            Value::Int(_) => Tag::Int,
            Value::Long(_) => Tag::Long,
            Value::Float(_) => Tag::Float,
            Value::Double(_) => Tag::Double,
            Value::Object(_) => Tag::Object,
        }
    }
}

/// A writable-only wrapper around [Value] that only writes the value itself
/// without a tag.
/// Used in places where JDWP specifies an `untagged-value` type and expects
/// no tag since it should be derived from context.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Untagged(Value);

impl Deref for Untagged {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl JdwpWritable for Untagged {
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        match self.0 {
            Value::Void => Ok(()),
            Value::Byte(v) => v.write(write),
            Value::Boolean(v) => v.write(write),
            Value::Char(v) => v.write(write),
            Value::Short(v) => v.write(write),
            Value::Int(v) => v.write(write),
            Value::Long(v) => v.write(write),
            Value::Float(v) => v.write(write),
            Value::Double(v) => v.write(write),
            Value::Object(v) => v.write(write),
        }
    }
}

macro_rules! tagged_io {
    ($enum:ident <-> $tag:ident, $($tpe:ident),* { $($read_extras:tt)* } { $($write_extras:tt)* }) => {
        impl JdwpReadable for $enum {
            fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
                match $tag::read(read)? {
                    $($tag::$tpe => JdwpReadable::read(read).map(Self::$tpe),)*
                    $($read_extras)*
                }
            }
        }

        impl JdwpWritable for $enum {
            fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
                match self {
                    $(Self::$tpe(v) => {
                        $tag::$tpe.write(write)?;
                        v.write(write)
                    },)*
                    $($write_extras)*
                }
            }
        }
    };
}

tagged_io! {
    Value <-> Tag,
    Byte, Boolean, Char, Int, Short, Long, Float, Double, Object
    {
        Tag::Void => Ok(Value::Void),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData))
    }
    { Self::Void => Ok(()) }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TaggedObjectID {
    /// an array object
    Array(ArrayID),
    /// an object
    Object(ObjectID),
    /// a String object
    String(StringID),
    /// a Thread object
    Thread(ThreadID),
    /// a ThreadGroup object
    ThreadGroup(ThreadGroupID),
    /// a ClassLoader object
    ClassLoader(ClassLoaderID),
    /// a class object object
    ClassObject(ClassObjectID),
}

impl TaggedObjectID {
    pub fn tag(self) -> Tag {
        use TaggedObjectID::*;
        match self {
            Array(_) => Tag::Array,
            Object(_) => Tag::Object,
            String(_) => Tag::String,
            Thread(_) => Tag::Thread,
            ThreadGroup(_) => Tag::ThreadGroup,
            ClassLoader(_) => Tag::ClassLoader,
            ClassObject(_) => Tag::ClassObject,
        }
    }

    pub fn decompose(self) -> (Tag, ObjectID) {
        (self.tag(), *self)
    }
}

impl Deref for TaggedObjectID {
    type Target = ObjectID;

    fn deref(&self) -> &Self::Target {
        use TaggedObjectID::*;
        match self {
            Array(id) => id,
            Object(id) => id,
            String(id) => id,
            Thread(id) => id,
            ThreadGroup(id) => id,
            ClassLoader(id) => id,
            ClassObject(id) => id,
        }
    }
}

impl Debug for TaggedObjectID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TaggedObjectID::*;
        match self {
            Array(id) => write!(f, "Array({})", id.0 .0),
            Object(id) => write!(f, "Object({})", id.0),
            String(id) => write!(f, "String({})", id.0 .0),
            Thread(id) => write!(f, "Thread({})", id.0 .0),
            ThreadGroup(id) => write!(f, "ThreadGroup({})", id.0 .0),
            ClassLoader(id) => write!(f, "ClassLoader({})", id.0 .0),
            ClassObject(id) => write!(f, "ClassObject({})", id.0 .0),
        }
    }
}

tagged_io! {
    TaggedObjectID <-> Tag,
    Array, Object, String, Thread, ThreadGroup, ClassLoader, ClassObject
    { _ => Err(io::Error::from(io::ErrorKind::InvalidData)) }
    {}
}

/// A compact representation of values used with some array operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayRegion {
    Byte(Vec<u8>),
    Boolean(Vec<bool>),
    Char(Vec<u16>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Object(Vec<TaggedObjectID>),
}

impl ArrayRegion {
    pub fn tag(&self) -> Tag {
        use ArrayRegion::*;
        match self {
            Byte(_) => Tag::Byte,
            Boolean(_) => Tag::Boolean,
            Char(_) => Tag::Char,
            Short(_) => Tag::Short,
            Int(_) => Tag::Int,
            Long(_) => Tag::Long,
            Float(_) => Tag::Float,
            Double(_) => Tag::Double,
            Object(_) => Tag::Object,
        }
    }

    pub fn len(&self) -> usize {
        use ArrayRegion::*;
        match self {
            Byte(v) => v.len(),
            Boolean(v) => v.len(),
            Char(v) => v.len(),
            Short(v) => v.len(),
            Int(v) => v.len(),
            Long(v) => v.len(),
            Float(v) => v.len(),
            Double(v) => v.len(),
            Object(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        use ArrayRegion::*;
        match self {
            Byte(v) => v.is_empty(),
            Boolean(v) => v.is_empty(),
            Char(v) => v.is_empty(),
            Short(v) => v.is_empty(),
            Int(v) => v.is_empty(),
            Long(v) => v.is_empty(),
            Float(v) => v.is_empty(),
            Double(v) => v.is_empty(),
            Object(v) => v.is_empty(),
        }
    }
}

tagged_io! {
    ArrayRegion <-> Tag,
    Byte, Boolean, Char, Short, Int, Long, Float, Double, Object
    { _ => Err(io::Error::from(io::ErrorKind::InvalidData)) }
    {}
}

/// A tagged representation of [ReferenceID], similar to how [TaggedObjectID]
/// is a representation of the [ObjectID].
///
/// This construct is not separated into a separate value type in JDWP spec and
/// exists only here in Rust, in JDWP it's usually represented by a pair of
/// [TypeTag] and [ReferenceID] values.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TaggedReferenceTypeID {
    /// a class reference
    Class(ClassID),
    /// an interface reference
    Interface(InterfaceID),
    /// an array reference
    Array(ArrayTypeID),
}

impl TaggedReferenceTypeID {
    pub fn tag(self) -> TypeTag {
        use TaggedReferenceTypeID::*;
        match self {
            Class(_) => TypeTag::Class,
            Interface(_) => TypeTag::Interface,
            Array(_) => TypeTag::Array,
        }
    }

    pub fn decompose(self) -> (TypeTag, ReferenceTypeID) {
        (self.tag(), *self)
    }
}

impl Deref for TaggedReferenceTypeID {
    type Target = ReferenceTypeID;

    fn deref(&self) -> &Self::Target {
        use TaggedReferenceTypeID::*;
        match self {
            Class(id) => &id.0,
            Interface(id) => &id.0,
            Array(id) => &id.0,
        }
    }
}

impl Debug for TaggedReferenceTypeID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TaggedReferenceTypeID::*;
        match self {
            Class(id) => write!(f, "Class({})", id.0 .0),
            Interface(id) => write!(f, "Interface({})", id.0 .0),
            Array(id) => write!(f, "Array({})", id.0 .0),
        }
    }
}

tagged_io! {
    TaggedReferenceTypeID <-> TypeTag,
    Class, Interface, Array
    {} {}
}

/// An executable location.
/// The location is identified by one byte type tag followed by a a class_id
/// followed by a method_id followed by an unsigned eight-byte index, which
/// identifies the location within the method. Index values are restricted as
/// follows:
///  - The index of the start location for the method is less than all other
///    locations in the method.
///  - The index of the end location for the method is greater than all other
///    locations in the method.
///  - If a line number table exists for a method, locations that belong to a
///    particular line must fall between the line's location index and the
///    location index of the next line in the table.
///
/// Index values within a method are monotonically increasing from the first
/// executable point in the method to the last. For many implementations, each
/// byte-code instruction in the method has its own index, but this is not
/// required.
///
/// The type tag is necessary to identify whether location's class_id
/// identifies a class or an interface. Almost all locations are within
/// classes, but it is possible to have executable code in the static
/// initializer of an interface.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct Location {
    reference_id: TaggedReferenceTypeID,
    method_id: MethodID,
    index: u64,
}

macro_rules! optional_tag_impl {
    ($($tpe:ident),* $(,)?) => {
        $(
            impl JdwpReadable for Option<$tpe> {
                fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
                    if read.peek_u8()? != 0 {
                        JdwpReadable::read(read).map(Some)
                    } else {
                        read.read_u8()?; // consume it
                        Ok(None)
                    }
                }
            }

            impl JdwpWritable for Option<$tpe> {
                fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
                    match self {
                        Some(x) => x.write(write),
                        None => write.write_u8(0),
                    }
                }
            }
        )*
    };
}

optional_tag_impl![Location, TaggedObjectID];

/// An opaque type for the request id, which is represented in JDWP docs as just a raw
/// integer and exists only here in Rust similar to all the other IDs.
#[derive(Debug, Copy, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct RequestID(i32);

impl RequestID {
    /// Creates a new instance of [RequestID] from an arbitrary number.
    /// # Safety
    /// It is up to the caller to ensure that the id does indeed exist and is correct.
    /// You can 'safely' obtain it by [JdwpReadable::read], but that will be incorrect obviously.
    pub const unsafe fn new(id: i32) -> Self {
        Self(id)
    }
}

/// Limit the requested event to be reported at most once after a given number
/// of occurrences.
///
/// The event is not reported the first count - 1 times this filter is reached.
///
/// To request a one-off event, call this method with a count of 1.
///
/// Once the count reaches 0, any subsequent filters in this request are
/// applied.
///
/// If none of those filters cause the event to be suppressed, the event is
/// reported.
///
/// Otherwise, the event is not reported.
///
/// In either case subsequent events are never reported for this request.
///
/// This modifier can be used with any event kind.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct Count {
    /// Count before event. One for one-off
    pub count: i32,
}

/// Conditional on expression
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct Conditional {
    /// For the future
    pub expr_id: i32,
}

/// Restricts reported events to those in the given thread.
/// This modifier can be used with any event kind except for class unload.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ThreadOnly {
    /// Required thread
    pub thread: ThreadID,
}

/// For class prepare events, restricts the events generated by this request to
/// be the preparation of the given reference type and any subtypes.
///
/// For monitor wait and waited events, restricts the events generated by this
/// request to those whose monitor object is of the given reference type or any
/// of its subtypes.
///
/// For other events, restricts the events generated by this request to those
/// whose location is in the given reference type or any of its subtypes.
///
/// An event will be generated for any location in a reference type that can be
/// safely cast to the given reference type.
///
/// This modifier can be used with any event kind except class unload, thread
/// start, and thread end.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ClassOnly {
    /// Required class
    pub class: ReferenceTypeID,
}

/// Restricts reported events to those for classes whose name matches the given
/// restricted regular expression.
///
/// For class prepare events, the prepared class name is matched.
///
/// For class unload events, the unloaded class name is matched.
///
/// For monitor wait and waited events, the name of the class of the monitor
/// object is matched.
///
/// For other events, the class name of the event's location is matched.
///
/// This modifier can be used with any event kind except thread start and
/// thread end.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ClassMatch {
    /// Required class pattern.
    ///
    /// Matches are limited to exact matches of the given class pattern and
    /// matches of patterns that begin or end with `*`; for example, `*.Foo` or
    /// `java.*`.
    pub class_pattern: String,
}

/// Restricts reported events to those for classes whose name does not match
/// the given restricted regular expression.
///
/// For class prepare events, the prepared class name is matched.
///
/// For class unload events, the unloaded class name is matched.
///
/// For monitor wait and waited events, the name of the class of the monitor
/// object is matched.
///
/// For other events, the class name of the event's location is matched.
///
/// This modifier can be used with any event kind except thread start and
/// thread end.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ClassExclude {
    /// Disallowed class pattern.
    ///
    /// Matches are limited to exact matches of the given class pattern and
    /// matches of patterns that begin or end with `*`; for example, `*.Foo` or
    /// `java.*`.
    pub class_pattern: String,
}

/// Restricts reported events to those that occur at the given location.
///
/// This modifier can be used with breakpoint, field access, field
/// modification, step, and exception event kinds.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct LocationOnly {
    /// Required location
    pub location: Location,
}

/// Restricts reported exceptions by their class and whether they are caught or
/// uncaught.
///
/// This modifier can be used with exception event kinds only.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct ExceptionOnly {
    /// Exception to report. `None` means report exceptions of all types.
    ///
    /// A non-null type restricts the reported exception events to exceptions
    /// of the given type or any of its subtypes.
    pub exception: Option<ReferenceTypeID>,
    /// Report caught exceptions
    pub uncaught: bool,
    /// Report uncaught exceptions.
    ///
    /// Note that it is not always possible to determine whether an exception
    /// is caught or uncaught at the time it is thrown.
    ///
    /// See the exception event catch location under composite events for more
    /// information.
    pub caught: bool,
}

/// Restricts reported events to those that occur for a given field.
///
/// This modifier can be used with field access and field modification event
/// kinds only.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct FieldOnly {
    /// Type in which field is declared
    pub declaring: ReferenceTypeID,
    /// Required field
    pub field_id: FieldID,
}

/// Restricts reported step events to those which satisfy depth and size
/// constraints.
///
/// This modifier can be used with step event kinds only.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct Step {
    /// Thread in which to step
    pub thread: ThreadID,
    /// Size of each step
    pub size: StepSize,
    /// Relative call stack limit
    pub depth: StepDepth,
}

/// Restricts reported events to those whose active 'this' object is the given
/// object.
///
/// Match value is the null object for static methods.
///
/// This modifier can be used with any event kind except class prepare,
/// class unload, thread start, and thread end.
///
/// Introduced in JDWP version 1.4.
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct InstanceOnly {
    /// Required 'this' object
    pub instance: ObjectID,
}

/// Restricts reported class prepare events to those for reference types which
/// have a source name which matches the given restricted regular expression.
///
/// The source names are determined by the reference type's
/// SourceDebugExtension.
///
/// This modifier can only be used with class prepare events.
///
/// Since JDWP version 1.6.
///
/// Requires the `can_use_source_name_filters` capability - see
/// [CapabilitiesNew].
#[derive(Debug, Clone, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct SourceNameMatch {
    /// Required source name pattern.
    /// Matches are limited to exact matches of the given pattern and matches
    /// of patterns that begin or end with `*`; for example, `*.Foo` or
    /// `java.*`
    pub source_name_pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    Count(Count),
    Conditional(Conditional),
    ThreadOnly(ThreadOnly),
    ClassOnly(ClassOnly),
    ClassMatch(ClassMatch),
    ClassExclude(ClassExclude),
    LocationOnly(LocationOnly),
    ExceptionOnly(ExceptionOnly),
    FieldOnly(FieldOnly),
    Step(Step),
    InstanceOnly(InstanceOnly),
    SourceNameMatch(SourceNameMatch),
}

tagged_io! {
    Modifier <-> ModifierKind,
    Count, Conditional, ThreadOnly, ClassOnly, ClassMatch, ClassExclude, LocationOnly, ExceptionOnly, FieldOnly, Step, InstanceOnly, SourceNameMatch
    {} {}
}
