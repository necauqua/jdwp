use jdwp_macros::JdwpReadable;

use crate::{
    codec::JdwpWritable,
    types::{MethodID, ReferenceTypeID},
};

use super::jdwp_command;

/// Returns line number information for the method, if present.
///
/// The line table maps source line numbers to the initial code index of the
/// line.
///
/// The line table is ordered by code index (from lowest to highest).
///
/// The line number information is constant unless a new class definition is
/// installed using [RedefineClasses](super::virtual_machine::RedefineClasses).
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
/// The variable table includes arguments and locals declared within the method.
/// For instance methods, the "this" reference is included in the table. Also,
/// synthetic variables may be present.
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
    /// The number of words in the frame used by arguments. Eight-byte arguments
    /// use two words; all others use one.
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
/// their bytecodes are the same except for indices into the constant pool and
/// the referenced constants are equal.
#[jdwp_command(bool, 6, 4)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct IsObsolete {
    /// The class.
    reference_type_id: ReferenceTypeID,
    /// The method.
    method_id: MethodID,
}

/// Returns variable information for the method, including generic signatures
/// for the variables.
///
/// The variable table includes arguments and locals declared within the method.
/// For instance methods, the "this" reference is included in the table. Also,
/// synthetic variables may be present. Generic signatures are described in the
/// signature attribute section in The Java™ Virtual Machine Specification.
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
    /// The number of words in the frame used by arguments. Eight-byte arguments
    /// use two words; all others use one.
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
