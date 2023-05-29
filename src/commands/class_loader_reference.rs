use jdwp_macros::{jdwp_command, JdwpWritable};

use crate::types::{ClassLoaderID, TaggedReferenceTypeID};

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
