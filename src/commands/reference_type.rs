use std::fmt::{self, Debug};

use super::jdwp_command;
use crate::{
    codec::{JdwpReadable, JdwpWritable},
    enums::ClassStatus,
    jvm::{FieldModifiers, MethodModifiers, TypeModifiers},
    types::{
        ClassLoaderID, ClassObjectID, FieldID, InterfaceID, MethodID, ReferenceTypeID,
        TaggedObjectID, TaggedReferenceTypeID, Value,
    },
};

/// Returns the JNI signature of a reference type.
///
/// JNI signature formats are described in the Java Native Interface
/// Specification.
///
/// For primitive classes the returned signature is the signature of the
/// corresponding primitive type; for example, "I" is returned as the signature
/// of the class represented by `java.lang.Integer.TYPE`.
#[jdwp_command(String, 2, 1)]
#[derive(Debug, JdwpWritable)]
pub struct Signature {
    /// The reference type ID
    ref_type: ReferenceTypeID,
}

/// Returns the instance of `java.lang.ClassLoader` which loaded a given
/// reference type.
///
/// If the reference type was loaded by the system class loader, the returned
/// object ID is null.
#[jdwp_command(Option<ClassLoaderID>, 2, 2)]
#[derive(Debug, JdwpWritable)]
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
/// `java.lang.Integer.TYPE`), the value of the returned bit mask is undefined.
#[jdwp_command(TypeModifiers, 2, 3)]
#[derive(Debug, JdwpWritable)]
pub struct Modifiers {
    ref_type: ReferenceTypeID,
}

/// Returns information for each field in a reference type.
///
/// Inherited fields are not included.
///
/// The field list will include any synthetic fields created by the compiler.
///
/// Fields are returned in the order they occur in the class file.
#[jdwp_command(Vec<Field>, 2, 4)]
#[derive(Debug, JdwpWritable)]
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
    /// In addition, the 0xf0000000 bit identifies the field as synthetic, if
    /// the synthetic attribute capability is available.
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
#[derive(Debug, JdwpWritable)]
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
    /// In addition, The 0xf0000000 bit identifies the method as synthetic, if
    /// the synthetic attribute capability is available.
    pub mod_bits: MethodModifiers,
}

/// Returns the value of one or more static fields of the reference type.
///
/// Each field must be member of the reference type or one of its superclasses,
/// superinterfaces, or implemented interfaces.
///
/// Access control is not enforced; for example, the values of private fields
/// can be obtained.
#[jdwp_command(Vec<Value>, 2, 6)]
#[derive(Debug, JdwpWritable)]
pub struct GetValues {
    pub ref_type: ReferenceTypeID,
    pub fields: Vec<FieldID>,
}

/// Returns the source file name in which a reference type was declared.
#[jdwp_command(String, 2, 7)]
#[derive(Debug, JdwpWritable)]
pub struct SourceFile {
    /// The reference type ID
    ref_type: ReferenceTypeID,
}

/// Returns the classes and interfaces directly nested within this type. Types
/// further nested within those types are not included.
#[jdwp_command(Vec<TaggedReferenceTypeID>, 2, 8)]
#[derive(Debug, JdwpWritable)]
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
/// If the class is initialized the INITIALIZED bit in the returned status bits
/// will be set.
///
/// If an error occurred during initialization then the ERROR bit in the
/// returned status bits will be set.
///
/// The returned status bits are undefined for array types and for primitive
/// classes (such as java.lang.Integer.TYPE).
#[jdwp_command(ClassStatus, 2, 9)]
#[derive(Debug, JdwpWritable)]
pub struct Status {
    /// The reference type ID
    ref_type: ReferenceTypeID,
}

/// Returns the interfaces declared as implemented by this class.
///
/// Interfaces indirectly implemented (extended by the implemented interface or
/// implemented by a superclass) are not included.
#[jdwp_command(Vec<InterfaceID>, 2, 10)]
#[derive(Debug, JdwpWritable)]
pub struct Interfaces {
    /// The reference type ID
    ref_type: ReferenceTypeID,
}

/// Returns the class object corresponding to this type.
#[jdwp_command(ClassObjectID, 2, 11)]
#[derive(Debug, JdwpWritable)]
pub struct ClassObject {
    /// The reference type ID
    ref_type: ReferenceTypeID,
}

/// Returns the value of the SourceDebugExtension attribute.
///
/// Since JDWP version 1.4. Requires canGetSourceDebugExtension capability -
/// see [CapabilitiesNew](super::virtual_machine::CapabilitiesNew).
#[jdwp_command(String, 2, 12)]
#[derive(Debug, JdwpWritable)]
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
#[derive(Debug, JdwpWritable)]
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

/// Returns information, including the generic signature if any, for each field
/// in a reference type.
///
/// Inherited fields are not included.
///
/// The field list will include any synthetic fields created by the compiler.
///
/// Fields are returned in the order they occur in the class file.
///
/// Generic signatures are described in the signature attribute section in
/// The Java™ Virtual Machine Specification.
///
/// Since JDWP version 1.5.

#[jdwp_command(Vec<FieldWithGeneric>, 2, 14)]
#[derive(Debug, JdwpWritable)]
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
    /// The generic signature of the field, or an empty string if there is none
    pub generic_signature: String,
    /// The modifier bit flags (also known as access flags) which provide
    /// additional information on the field declaration.
    ///
    /// Individual flag values are defined in Chapter 4 of The Java™ Virtual
    /// Machine Specification.
    ///
    /// In addition, the 0xf0000000 bit identifies the field as synthetic, if
    /// the synthetic attribute capability is available.
    pub mod_bits: FieldModifiers,
}

/// Returns information, including the generic signature if any, for each method
/// in a reference type. Inherited methodss are not included.
/// The list of methods will include constructors (identified with the name
/// "&lt;init>"), the initialization method (identified with the name
/// "&lt;clinit>") if present, and any synthetic methods created by the
/// compiler. Methods are returned in the order they occur in the class file.
///
/// Generic signatures are described in the signature attribute section in
/// The Java™ Virtual Machine Specification.
///
/// Since JDWP version 1.5.
#[jdwp_command(Vec<MethodWithGeneric>, 2, 15)]
#[derive(Debug, JdwpWritable)]
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
    /// The generic signature of the method, or an empty string if there is none
    pub generic_signature: String,
    /// The modifier bit flags (also known as access flags) which provide
    /// additional information on the method declaration.
    ///
    /// Individual flag values are defined in Chapter 4 of The Java™ Virtual
    /// Machine Specification.
    ///
    /// In addition, the 0xf0000000 bit identifies the method as synthetic, if
    /// the synthetic attribute capability is available.
    pub mod_bits: MethodModifiers,
}

/// Returns instances of this reference type.
///
/// Only instances that are reachable for the purposes of garbage collection are
/// returned.
#[jdwp_command(Vec<TaggedObjectID>, 2, 16)]
#[derive(Debug, JdwpWritable)]
pub struct Instances {
    /// The reference type ID
    ref_type: ReferenceTypeID,
    /// Maximum number of instances to return.
    ///
    /// Must be non-negative.
    ///
    /// If zero, all instances are returned.
    max_instances: u32,
}

/// Returns the class object corresponding to this type.
#[jdwp_command(2, 17)]
#[derive(Debug, JdwpWritable)]
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
#[derive(Debug, JdwpWritable)]
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
    /// Raw bytes of constant pool
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
