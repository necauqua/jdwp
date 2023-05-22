use std::fmt::Debug;

use jdwp_macros::jdwp_command;

use crate::{
    codec::{JdwpReadable, JdwpWritable},
    types::ClassObjectID,
    CommandId,
};

use crate::{
    enums::Tag,
    types::{ArrayID, ArrayTypeID, ClassLoaderID, ObjectID, TaggedReferenceTypeID},
};

pub mod array_type {
    use super::*;

    /// Creates a new array object of this type with a given length.
    #[jdwp_command(4, 1)]
    #[derive(Debug, JdwpWritable)]
    pub struct NewInstance {
        /// The array type of the new instance
        array_type_id: ArrayTypeID,
        /// The length of the array
        length: i32,
    }

    #[derive(Debug, JdwpReadable)]
    pub struct NewInstanceReply {
        // should always be Tag::Array
        _tag: Tag,
        /// The newly created array object
        pub new_array: ArrayID,
    }
}

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
    #[derive(Debug, JdwpWritable)]
    pub struct VisibleClasses {
        /// The class loader object ID
        class_loader_id: ClassLoaderID,
    }
}

pub mod class_object_reference {
    use super::*;

    /// Returns the reference type reflected by this class object.
    #[jdwp_command(TaggedReferenceTypeID, 17, 1)]
    #[derive(Debug, JdwpWritable)]
    pub struct ReflectedType {
        /// The class object
        class_object_id: ClassObjectID,
    }
}

pub mod string_reference {
    use super::*;

    /// Returns the characters contained in the string.
    #[jdwp_command(String, 10, 1)]
    #[derive(Debug, JdwpWritable)]
    pub struct Value {
        /// The String object ID
        string_object: ObjectID,
    }
}

/// This module is defined to mirror the JDWP command set, which is empty
pub mod field {}

/// This module is defined to mirror the JDWP command set, which is empty
pub mod interface_type {}

pub mod array_reference;
pub mod class_type;
pub mod event;
pub mod event_request;
pub mod method;
pub mod object_reference;
pub mod reference_type;
pub mod stack_frame;
pub mod thread_group_reference;
pub mod thread_reference;
pub mod virtual_machine;

pub trait Command {
    const ID: CommandId;

    type Output;
}
