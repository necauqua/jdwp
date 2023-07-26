use crate::codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter};
use bitflags::bitflags;
use std::{
    borrow::Cow,
    io::{self, Read, Write},
    rc::Rc,
};
use thiserror::Error;

use byteorder::{ReadBytesExt, BE};

/// This represents an item in the constant pool table.
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ConstantPoolItem {
    /// Used as a placeholder taking the second slot for longs and doubles, as
    /// well as the 0th slot in the vec of constant pool items for indexing to
    /// work nicely.
    Stub = 0,
    /// A UTF-8 string, decoded from the modified UTF-8 used by Java
    Utf8(String) = 1,
    /// An signed 32bit integer
    Integer(i32) = 3,
    /// A 32bit single-precision IEEE 754 floating point number
    Float(f32) = 4,
    /// A signed 64bit integer
    ///
    /// Always followed by a `ConstantPoolItem::Stub`
    Long(i64) = 5,
    /// A 64bit double-precision IEEE 754 floating point number
    ///
    /// Always followed by a `ConstantPoolItem::Stub`
    Double(f64) = 6,
    /// Class reference: an index within the constant pool to a UTF-8 string
    /// containing the fully qualified class name (in internal format)
    Class(u16) = 7,
    /// String reference: an index within the constant pool to a UTF-8 string
    String(u16) = 8,
    /// Field reference: two indexes within the constant pool, the first
    /// pointing to a Class reference, the second to a Name and Type descriptor.
    Fieldref {
        class_index: u16,
        name_and_type_index: u16,
    } = 9,
    /// Method reference: two indexes within the constant pool, the first
    /// pointing to a Class reference, the second to a Name and Type
    /// descriptor.
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    } = 10,
    /// Interface method reference: two indexes within the constant pool, the
    /// first pointing to a Class reference, the second to a Name and Type
    /// descriptor.
    InterfaceMethodref {
        class_index: u16,
        name_and_type_index: u16,
    } = 11,
    /// Name and type descriptor: two indexes to UTF-8 strings within the
    /// constant pool, the first representing a name (identifier) and the second
    /// a specially encoded type descriptor.
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    } = 12,
    /// This structure is used to represent a method handle and consists of one
    /// byte of type descriptor, followed by an index within the constant pool.
    MethodHandle {
        reference_kind: ReferenceKind,
        reference_index: u16,
    } = 15,
    /// This structure is used to represent a method type, and consists of an
    /// index within the constant pool.
    MethodType(u16) = 16,
    /// This is used to specify a dynamically computed constant produced by
    /// invocation of a bootstrap method.
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    } = 17,
    /// This is used by an invokedynamic instruction to specify a bootstrap
    /// method, the dynamic invocation name, the argument and return types of
    /// the call, and optionally, a sequence of additional constants called
    /// static arguments to the bootstrap method.
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    } = 18,
    /// This is used to identify a module.
    Module(u16) = 19,
    /// This is used to identify a package exported or opened by a module.
    Package(u16) = 20,
}

#[derive(Debug, Error)]
pub enum ConstantPoolParsingError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Failed to decode a string at index {index}")]
    BadUtf8 { index: u32 },
    #[error("Unknown constant pool tag: {tag} at index {index}")]
    BadTag { tag: u8, index: u32 },
    #[error("Unknown reference kind: {kind} at index {index}")]
    BadReferenceKind { kind: u8, index: u32 },
}

impl ConstantPoolItem {
    fn read(index: u32, mut read: impl Read) -> Result<Self, ConstantPoolParsingError> {
        let tag = read.read_u8()?;
        let item = match tag {
            1 => {
                let length = read.read_u16::<BE>()?;
                let mut bytes = vec![0; length as usize];
                read.read_exact(&mut bytes)?;

                let cow = cesu8::from_java_cesu8(&bytes)
                    .map_err(|_| ConstantPoolParsingError::BadUtf8 { index })?;

                ConstantPoolItem::Utf8(match cow {
                    Cow::Borrowed(_) => {
                        // SAFETY: from_cesu8 only returns borrowed if input was valid utf-8
                        // so we just reinterpret to avoid the clone in Cow::into_owned
                        // Maybe I should make a PR to add this to cesu8?
                        unsafe { String::from_utf8_unchecked(bytes) }
                    }
                    Cow::Owned(s) => s,
                })
            }
            3 => ConstantPoolItem::Integer(read.read_i32::<BE>()?),
            4 => ConstantPoolItem::Float(read.read_f32::<BE>()?),
            5 => ConstantPoolItem::Long(read.read_i64::<BE>()?),
            6 => ConstantPoolItem::Double(read.read_f64::<BE>()?),
            7 => ConstantPoolItem::Class(read.read_u16::<BE>()?),
            8 => ConstantPoolItem::String(read.read_u16::<BE>()?),
            9 => ConstantPoolItem::Fieldref {
                class_index: read.read_u16::<BE>()?,
                name_and_type_index: read.read_u16::<BE>()?,
            },
            10 => ConstantPoolItem::Methodref {
                class_index: read.read_u16::<BE>()?,
                name_and_type_index: read.read_u16::<BE>()?,
            },
            11 => ConstantPoolItem::InterfaceMethodref {
                class_index: read.read_u16::<BE>()?,
                name_and_type_index: read.read_u16::<BE>()?,
            },
            12 => ConstantPoolItem::NameAndType {
                name_index: read.read_u16::<BE>()?,
                descriptor_index: read.read_u16::<BE>()?,
            },
            15 => {
                use ReferenceKind::*;

                ConstantPoolItem::MethodHandle {
                    reference_kind: match read.read_u8()? {
                        1 => GetField,
                        2 => GetStatic,
                        3 => PutField,
                        4 => PutStatic,
                        5 => InvokeVirtual,
                        6 => InvokeStatic,
                        7 => InvokeSpecial,
                        8 => NewInvokeSpecial,
                        9 => InvokeInterface,
                        kind => {
                            return Err(ConstantPoolParsingError::BadReferenceKind { kind, index })
                        }
                    },
                    reference_index: read.read_u16::<BE>()?,
                }
            }
            16 => ConstantPoolItem::MethodType(read.read_u16::<BE>()?),
            17 => ConstantPoolItem::Dynamic {
                bootstrap_method_attr_index: read.read_u16::<BE>()?,
                name_and_type_index: read.read_u16::<BE>()?,
            },
            18 => ConstantPoolItem::InvokeDynamic {
                bootstrap_method_attr_index: read.read_u16::<BE>()?,
                name_and_type_index: read.read_u16::<BE>()?,
            },
            19 => ConstantPoolItem::Module(read.read_u16::<BE>()?),
            20 => ConstantPoolItem::Package(read.read_u16::<BE>()?),
            _ => return Err(ConstantPoolParsingError::BadTag { tag, index }),
        };
        Ok(item)
    }

    /// Reads the constant pool from the given reader.
    ///
    /// Note that the count is number of items plus one (!), and the first item
    /// will always be a `ConstantPoolItem::Stub`.
    ///
    /// This is made in accordance with the JVM specification, and to make
    /// indexing easier.
    ///
    /// A stub is also added after each long and double, so that the resulting
    /// vector completely mimics the constant pool indexing.
    pub fn read_all(
        count: u32,
        mut read: impl Read,
    ) -> Result<Vec<Self>, ConstantPoolParsingError> {
        let mut result = Vec::with_capacity(count as usize);
        result.push(ConstantPoolItem::Stub);

        while (result.len() as u32) < count {
            match Self::read(result.len() as u32, &mut read)? {
                item @ (ConstantPoolItem::Long(_) | ConstantPoolItem::Double(_)) => {
                    result.extend([item, ConstantPoolItem::Stub])
                }
                item => result.push(item),
            }
        }
        Ok(result)
    }
}

/// Type of item that can be referenced by a constant pool index.
#[derive(Debug)]
pub enum IndexableItem {
    Utf8,
    Class,
    NameAndType,
    Fieldref,
    Methodref,
}

/// Error that can occur when resolving a constant pool index.
#[derive(Debug, Error)]
#[error("No {item:?} item at index {index}")]
pub struct ResolutionError {
    pub index: u16,
    pub item: IndexableItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameAndType {
    pub name: Rc<str>,
    pub descriptor: Rc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    pub class: Rc<str>,
    pub name: Rc<str>,
    pub descriptor: Rc<str>,
}

impl Ref {
    pub fn new(class: Rc<str>, name_and_type: NameAndType) -> Self {
        Self {
            class,
            name: name_and_type.name,
            descriptor: name_and_type.descriptor,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    GetField = 1,
    GetStatic = 2,
    PutField = 3,
    PutStatic = 4,
    InvokeVirtual = 5,
    InvokeStatic = 6,
    InvokeSpecial = 7,
    NewInvokeSpecial = 8,
    InvokeInterface = 9,
}

/// This represents a constant pool structure with all the indexes resolved
#[derive(Debug, Clone)]
pub enum ConstantPoolValue {
    Utf8(Rc<str>),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(Rc<str>),
    String(Rc<str>),
    Fieldref(Ref),
    Methodref(Ref),
    InterfaceMethodref(Ref),
    NameAndType(NameAndType),
    MethodHandle {
        reference_kind: ReferenceKind,
        reference: Ref,
    },
    MethodType(Rc<str>),
    Dynamic {
        bootstrap_method_attr_index: u16,
        name: Rc<str>,
        descriptor: Rc<str>,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name: Rc<str>,
        descriptor: Rc<str>,
    },
    Module(Rc<str>),
    Package(Rc<str>),
}

macro_rules! resolve_methods {
    ($(fn $name:ident($pool:ident, $items:ident, $variant:ident $bindings:tt { $($constructor:tt)* }) -> $res:ty;)*) => {
        $(
            pub fn $name(
                $pool: &mut [Option<Self>],
                $items: &[ConstantPoolItem],
                index: u16
            ) -> Result<$res, ResolutionError> {
                match $pool.get(index as usize).and_then(Option::as_ref)  {
                    Some(Self::$variant(thing)) => return Ok(thing.clone()),
                    None => {
                        if let Some(ConstantPoolItem::$variant $bindings) = $items.get(index as usize) {
                            let thing = $($constructor)*;
                            $pool[index as usize] = Some(Self::$variant(thing.clone()));
                            return Ok(thing);
                        }
                    }
                    _ => {},
                }
                Err(ResolutionError {
                    index,
                    item: IndexableItem::$variant,
                })
            }
        )*
    };
}

impl ConstantPoolValue {
    resolve_methods! {

        fn resolve_string(pool, items, Utf8(string) { Rc::<str>::from(string.as_ref()) }) -> Rc<str>;

        fn resolve_class(pool, items, Class(string_index) { Self::resolve_string(pool, items, *string_index)? }) -> Rc<str>;

        fn resolve_name_and_type(pool, items, NameAndType { name_index, descriptor_index } {
            NameAndType {
                name: Self::resolve_string(pool, items, *name_index)?,
                descriptor: Self::resolve_string(pool, items, *descriptor_index)?,
            }
        }) -> NameAndType;

        fn resolve_fieldref(pool, items, Fieldref { class_index, name_and_type_index } {
            Self::resolve_ref(pool, items, *class_index, *name_and_type_index)?
        }) -> Ref;
    }

    // this is different from other resolve methods because we bunch in the
    // interface methodref
    fn resolve_methodref(
        pool: &mut [Option<ConstantPoolValue>],
        items: &[ConstantPoolItem],
        index: u16,
    ) -> Result<Ref, ResolutionError> {
        match pool.get(index as usize).and_then(Option::as_ref) {
            Some(ConstantPoolValue::Methodref(r)) => return Ok(r.clone()),
            Some(ConstantPoolValue::InterfaceMethodref(r)) => return Ok(r.clone()),
            None => match items.get(index as usize) {
                Some(ConstantPoolItem::Methodref {
                    class_index,
                    name_and_type_index,
                }) => {
                    let r = Self::resolve_ref(pool, items, *class_index, *name_and_type_index)?;
                    pool[index as usize] = Some(ConstantPoolValue::Methodref(r.clone()));
                    return Ok(r);
                }
                Some(ConstantPoolItem::InterfaceMethodref {
                    class_index,
                    name_and_type_index,
                }) => {
                    let r = Self::resolve_ref(pool, items, *class_index, *name_and_type_index)?;
                    pool[index as usize] = Some(ConstantPoolValue::InterfaceMethodref(r.clone()));
                    return Ok(r);
                }
                _ => {}
            },
            _ => {}
        }
        Err(ResolutionError {
            index,
            item: IndexableItem::Methodref,
        })
    }

    fn resolve_ref(
        pool: &mut [Option<ConstantPoolValue>],
        items: &[ConstantPoolItem],
        class_index: u16,
        name_and_type_index: u16,
    ) -> Result<Ref, ResolutionError> {
        Ok(Ref::new(
            Self::resolve_class(pool, items, class_index)?,
            Self::resolve_name_and_type(pool, items, name_and_type_index)?,
        ))
    }

    /// Assuming given slice is a complete constant pool, resolves all the
    /// indexes such that the resulting [ConstantPoolValue]s do not require any
    /// further lookups.
    ///
    /// This method supports forward references, meaning it will resolve an item
    /// that references an item at a greater index that has not been resolved
    /// yet.
    ///
    /// The resulting vector is just a list, and the [ConstantPoolValue] does
    /// not have a stub variant.
    pub fn resolve(items: &[ConstantPoolItem]) -> Result<Vec<Self>, ResolutionError> {
        use ConstantPoolItem as Item;
        use ConstantPoolValue as Value;

        let mut pool = Vec::with_capacity(items.len());

        // Option<ConstantPoolValue> is not Default :(
        for _ in 0..items.len() {
            pool.push(None);
        }

        for (index, item) in items.iter().enumerate() {
            match item {
                Item::Stub => {}
                Item::Utf8(string) => {
                    if pool[index].is_some() {
                        // was already resolved by a forward reference
                        continue;
                    }
                    pool[index] = Some(Value::Utf8(Rc::<str>::from(string.as_ref())));
                }
                Item::Integer(value) => {
                    pool[index] = Some(Value::Integer(*value));
                }
                Item::Float(value) => {
                    pool[index] = Some(Value::Float(*value));
                }
                Item::Long(value) => {
                    pool[index] = Some(Value::Long(*value));
                }
                Item::Double(value) => {
                    pool[index] = Some(Value::Double(*value));
                }
                Item::Class(idx) => {
                    if pool[index].is_some() {
                        continue;
                    }
                    pool[index] = Some(Value::Class(Self::resolve_string(&mut pool, items, *idx)?));
                }
                Item::String(idx) => {
                    pool[index] =
                        Some(Value::String(Self::resolve_string(&mut pool, items, *idx)?));
                }
                Item::Fieldref {
                    class_index,
                    name_and_type_index,
                } => {
                    if pool[index].is_some() {
                        continue;
                    }
                    pool[index] = Some(Value::Fieldref(Self::resolve_ref(
                        &mut pool,
                        items,
                        *class_index,
                        *name_and_type_index,
                    )?));
                }
                Item::Methodref {
                    class_index,
                    name_and_type_index,
                } => {
                    if pool[index].is_some() {
                        continue;
                    }
                    pool[index] = Some(Value::Methodref(Self::resolve_ref(
                        &mut pool,
                        items,
                        *class_index,
                        *name_and_type_index,
                    )?));
                }
                Item::InterfaceMethodref {
                    class_index,
                    name_and_type_index,
                } => {
                    if pool[index].is_some() {
                        continue;
                    }
                    pool[index] = Some(Value::InterfaceMethodref(Self::resolve_ref(
                        &mut pool,
                        items,
                        *class_index,
                        *name_and_type_index,
                    )?));
                }
                Item::NameAndType {
                    name_index,
                    descriptor_index,
                } => {
                    if pool[index].is_some() {
                        continue;
                    }
                    pool[index] = Some(Value::NameAndType(NameAndType {
                        name: Self::resolve_string(&mut pool, items, *name_index)?,
                        descriptor: Self::resolve_string(&mut pool, items, *descriptor_index)?,
                    }));
                }
                Item::MethodHandle {
                    reference_kind,
                    reference_index,
                } => {
                    use ReferenceKind::*;

                    let reference = match reference_kind {
                        GetField | GetStatic | PutField | PutStatic => {
                            Self::resolve_fieldref(&mut pool, items, *reference_index)?
                        }
                        _ => Self::resolve_methodref(&mut pool, items, *reference_index)?,
                    };

                    pool[index] = Some(Value::MethodHandle {
                        reference_kind: *reference_kind,
                        reference,
                    });
                }
                Item::MethodType(descriptor_index) => {
                    pool[index] = Some(Value::MethodType(Self::resolve_string(
                        &mut pool,
                        items,
                        *descriptor_index,
                    )?));
                }
                Item::Dynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                } => {
                    let NameAndType { name, descriptor } =
                        Self::resolve_name_and_type(&mut pool, items, *name_and_type_index)?;
                    pool[index] = Some(Value::Dynamic {
                        bootstrap_method_attr_index: *bootstrap_method_attr_index,
                        name,
                        descriptor,
                    });
                }
                Item::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                } => {
                    let NameAndType { name, descriptor } =
                        Self::resolve_name_and_type(&mut pool, items, *name_and_type_index)?;
                    pool[index] = Some(Value::InvokeDynamic {
                        bootstrap_method_attr_index: *bootstrap_method_attr_index,
                        name,
                        descriptor,
                    });
                }
                Item::Module(idx) => {
                    pool[index] =
                        Some(Value::Module(Self::resolve_string(&mut pool, items, *idx)?));
                }
                Item::Package(idx) => {
                    pool[index] = Some(Value::Package(Self::resolve_string(
                        &mut pool, items, *idx,
                    )?));
                }
            }
        }

        // only stubs should be None, and flatten will remove them
        Ok(pool.into_iter().flatten().collect())
    }
}

// Access flags are not specified in the JDWP protocol, so they are in the JVM
// module.
// However, those bitflags are for convenience, and they can store any
// value

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct TypeModifiers: u32 {
        /// Declared public; may be accessed from outside its package.
        const PUBLIC = 0x0001;
        /// Declared final; no subclasses allowed.
        const FINAL = 0x0010;
        /// Treat superclass methods specially when invoked by the invokespecial instruction.
        const SUPER = 0x0020;
        /// Is an interface, not a class.
        const INTERFACE = 0x0200;
        /// Declared abstract; must not be instantiated.
        const ABSTRACT = 0x0400;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC = 0x1000;
        /// Declared as an annotation type.
        const ANNOTATION = 0x2000;
        /// Declared as an enum type.
        const ENUM = 0x4000;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct FieldModifiers: u32 {
        /// Declared public; may be accessed from outside its package.
        const PUBLIC = 0x0001;
        /// Declared private; usable only within the defining class.
        const PRIVATE = 0x0002;
        /// Declared protected; may be accessed within subclasses.
        const PROTECTED = 0x0004;
        /// Declared static.
        const STATIC = 0x0008;
        /// Declared final; never directly assigned to after object
        /// construction (JLS §17.5).
        const FINAL = 0x0010;
        /// Declared volatile; cannot be cached.
        const VOLATILE = 0x0040;
        /// Declared transient; not written or read by a persistent object
        /// manager.
        const TRANSIENT = 0x0080;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC = 0x1000;
        /// Declared as an element of an enum.
        const ENUM = 0x4000;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MethodModifiers: u32 {
        /// Declared public; may be accessed from outside its package.
        const PUBLIC = 0x0001;
        /// Declared private; accessible only within the defining class.
        const PRIVATE = 0x0002;
        /// Declared protected; may be accessed within subclasses.
        const PROTECTED = 0x0004;
        /// Declared static.
        const STATIC = 0x0008;
        /// Declared final; must not be overridden (§5.4.5).
        const FINAL = 0x0010;
        /// Declared synchronized; invocation is wrapped by a monitor use.
        const SYNCHRONIZED = 0x0020;
        /// A bridge method, generated by the compiler.
        const BRIDGE = 0x0040;
        /// Declared with variable number of arguments.
        const VARARGS = 0x0080;
        /// Declared native; implemented in a language other than Java.
        const NATIVE = 0x0100;
        /// Declared abstract; no implementation is provided.
        const ABSTRACT = 0x0400;
        /// Declared strictfp; floating-point mode is FP-strict.
        const STRICT = 0x0800;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC = 0x1000;
    }
}

macro_rules! jdwp_access_flags {
    ($($types:ident),*) => {
        $(
            impl JdwpReadable for $types {
                fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
                    u32::read(read).map(Self::from_bits_retain)
                }
            }

            impl JdwpWritable for $types {
                fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
                    self.bits().write(write)
                }
            }
        )*
    };
}

jdwp_access_flags![TypeModifiers, FieldModifiers, MethodModifiers];
