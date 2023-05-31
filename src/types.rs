use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    enums::{Tag, TypeTag},
};
use std::{
    fmt::{Debug, Formatter},
    io::{self, Read, Write},
    ops::Deref,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub trait JdwpId: Clone + Copy {
    type Raw;

    /// Creates an instance of Self from an arbitrary number.
    ///
    /// This is not unsafe as invalid IDs do not cause UB, but it is up to the
    /// caller to ensure that the id is valid for the target JVM.
    ///
    /// Also nothing prevents you from getting an ID from one JVM and using it
    /// for another, or creating it through [JdwpReadable::read].
    fn from_raw(raw: Self::Raw) -> Self;

    /// The underlying raw value.
    ///
    /// It is opaque and given out by the JVM, but in case you ever need it,
    /// here it is.
    fn raw(self) -> Self::Raw;
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
/// Garbage collection can be disabled with the
/// [DisableCollection](super::commands::object_reference::DisableCollection)
/// command, but it is not usually necessary to do so.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ObjectID(u64);

/// Uniquely identifies a method in some class in the target VM.
///
/// The [MethodID] must uniquely identify the method within its class/interface
/// or any of its subclasses/subinterfaces/implementors.
///
/// A [MethodID] is not necessarily unique on its own; it is always paired with
/// a [ReferenceTypeID] to uniquely identify one method.
///
/// The [ReferenceTypeID] can identify either the declaring type of the method
/// or a subtype.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
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
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FieldID(u64);

/// Uniquely identifies a frame in the target VM.
///
/// The [FrameID] must uniquely identify the frame within the entire VM (not
/// only within a given thread).
///
/// The [FrameID] need only be valid during the time its thread is suspended.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
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
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ReferenceTypeID(u64);

macro_rules! ids {
    ($($id:ident: $tpe:ident),* $(,)?) => {
        $(
            impl JdwpId for $tpe {
                type Raw = u64;

                fn from_raw(raw: u64) -> Self {
                    Self(raw)
                }

                fn raw(self) -> u64 {
                    self.0
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
    field_id_size: FieldID,
    method_id_size: MethodID,
    object_id_size: ObjectID,
    reference_type_id_size: ReferenceTypeID,
    frame_id_size: FrameID,
}

/// Uniquely identifies an object in the target VM that is known to be a thread.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct ThreadID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a thread
/// group.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct ThreadGroupID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a string
/// object.
///
/// Note: this is very different from string, which is a value.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct StringID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a class
/// loader object.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct ClassLoaderID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be a class
/// object.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct ClassObjectID(ObjectID);

/// Uniquely identifies an object in the target VM that is known to be an array.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct ArrayID(ObjectID);

/// Uniquely identifies a reference type in the target VM that is known to be
/// a class type.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct ClassID(ReferenceTypeID);

/// Uniquely identifies a reference type in the target VM that is known to be
/// an interface type.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct InterfaceID(ReferenceTypeID);

/// Uniquely identifies a reference type in the target VM that is known to be
/// an array type.
#[derive(Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
#[repr(transparent)]
pub struct ArrayTypeID(ReferenceTypeID);

macro_rules! wrapper_ids {
    ($($deref:ident {$($tpe:ident),* $(,)?})*) => {
        $($(
            impl JdwpId for $tpe {
                type Raw = u64;

                fn from_raw(raw: u64) -> Self {
                    Self($deref::from_raw(raw))
                }

                fn raw(self) -> u64 {
                    self.0.0
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
///
/// Used in places where JDWP specifies an `untagged-value` type and expects
/// no tag since it should be derived from context.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UntaggedValue(pub Value);

impl From<Value> for UntaggedValue {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl Deref for UntaggedValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl JdwpWritable for UntaggedValue {
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

macro_rules! tagged_jdwp_io {
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
    ($($tt:tt)*) => { tagged_jdwp_io!($($tt)* {} {}); }
}

tagged_jdwp_io! {
    Value <-> Tag,
    Byte, Boolean, Char, Int, Short, Long, Float, Double, Object
    {
        Tag::Void => Ok(Value::Void),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData))
    }
    { Self::Void => Ok(()) }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

tagged_jdwp_io! {
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

tagged_jdwp_io! {
    ArrayRegion <-> Tag,
    Byte, Boolean, Char, Short, Int, Long, Float, Double, Object
    { _ => Err(io::Error::from(io::ErrorKind::InvalidData)) }
    {}
}

/// A tagged representation of [ReferenceTypeID], similar to how
/// [TaggedObjectID] is a representation of the [ObjectID].
///
/// This construct is not separated into a separate value type in JDWP spec and
/// exists only here in Rust, in JDWP it's usually represented by a pair of
/// [TypeTag] and [ReferenceTypeID] values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

tagged_jdwp_io! {
    TaggedReferenceTypeID <-> TypeTag,
    Class, Interface, Array
}

/// An executable location.
///
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
pub struct Location {
    pub reference_id: TaggedReferenceTypeID,
    pub method_id: MethodID,
    pub index: u64,
}

impl JdwpReadable for Option<TaggedReferenceTypeID> {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        use TaggedReferenceTypeID::*;

        let Some(tag) = Option::<TypeTag>::read(read)? else { return Ok(None); };
        let id = ReferenceTypeID::read(read)?;
        let res = match tag {
            TypeTag::Class => Class(ClassID(id)),
            TypeTag::Interface => Interface(InterfaceID(id)),
            TypeTag::Array => Array(ArrayTypeID(id)),
        };
        Ok(Some(res))
    }
}

impl JdwpReadable for Option<Location> {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        let Some(reference_id) = Option::<TaggedReferenceTypeID>::read(read)? else { return Ok(None); };
        let res = Location {
            reference_id,
            method_id: JdwpReadable::read(read)?,
            index: JdwpReadable::read(read)?,
        };
        Ok(Some(res))
    }
}

impl JdwpReadable for Option<Value> {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        use JdwpReadable as R;
        use Value::*;

        let Some(tag) = Option::<Tag>::read(read)? else { return Ok(None); };
        let res = match tag {
            Tag::Byte => Byte(R::read(read)?),
            Tag::Char => Char(R::read(read)?),
            Tag::Short => Short(R::read(read)?),
            Tag::Int => Int(R::read(read)?),
            Tag::Long => Long(R::read(read)?),
            Tag::Float => Float(R::read(read)?),
            Tag::Double => Double(R::read(read)?),
            Tag::Object => Object(R::read(read)?),
            Tag::Boolean => Boolean(R::read(read)?),
            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        };
        Ok(Some(res))
    }
}

impl JdwpReadable for Option<TaggedObjectID> {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        use JdwpReadable as R;
        use TaggedObjectID::*;

        let Some(tag) = Option::<Tag>::read(read)? else { return Ok(None); };
        let res = match tag {
            Tag::Array => Array(R::read(read)?),
            Tag::Object => Object(R::read(read)?),
            Tag::String => String(R::read(read)?),
            Tag::Thread => Thread(R::read(read)?),
            Tag::ThreadGroup => ThreadGroup(R::read(read)?),
            Tag::ClassLoader => ClassLoader(R::read(read)?),
            Tag::ClassObject => ClassObject(R::read(read)?),
            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        };
        Ok(Some(res))
    }
}

/// An opaque type for the request id, which is represented in JDWP docs as just
/// a raw integer and exists only here in Rust similar to all the other IDs.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, JdwpReadable, JdwpWritable)]
pub struct RequestID(i32);

impl JdwpId for RequestID {
    type Raw = i32;

    fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    fn raw(self) -> i32 {
        self.0
    }
}

impl JdwpReadable for Option<RequestID> {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        match i32::read(read)? {
            0 => Ok(None),
            x => Ok(Some(RequestID(x))),
        }
    }
}

/// A response from 3 types of invoke method commands: virtual, static and
/// interface static. The response is either a value or an exception - we model
/// it with a enum for better ergonomics.
#[derive(Debug)]
pub enum InvokeMethodReply {
    Value(Value),
    Exception(TaggedObjectID),
}

impl JdwpReadable for InvokeMethodReply {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        let value = Option::<Value>::read(read)?;
        let exception = Option::<TaggedObjectID>::read(read)?;
        match (value, exception) {
            (Some(value), None) => Ok(InvokeMethodReply::Value(value)),
            (None, Some(exception)) => Ok(InvokeMethodReply::Exception(exception)),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }
}
