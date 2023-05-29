use std::io::{self, Read};

use jdwp_macros::jdwp_command;

use crate::{codec::JdwpReader, enums::InvokeOptions, types::*};

use super::{JdwpReadable, JdwpWritable};

/// Returns the immediate superclass of a class.
///
/// The return is null if the class is java.lang.Object.
#[jdwp_command(Option<ClassID>, 3, 1)]
#[derive(Debug, JdwpWritable)]
pub struct Superclass {
    /// The class type ID.
    class_id: ClassID,
}

/// Sets the value of one or more static fields.
///
/// Each field must be member of the class type or one of its superclasses,
/// superinterfaces, or implemented interfaces.
///
/// Access control is not enforced; for example, the values of private fields
/// can be set.
///
/// Final fields cannot be set.
///
/// For primitive values, the value's type must match the field's type exactly.
///
/// For object values, there must exist a widening reference conversion from the
/// value's type to thefield's type and the field's type must be loaded.
#[jdwp_command((), 3, 2)]
#[derive(Debug, JdwpWritable)]
pub struct SetValues {
    /// The class type ID.
    class_id: ClassID,
    /// Fields to set and their values.
    values: Vec<(FieldID, UntaggedValue)>,
}

/// Invokes a static method. The method must be member of the class type or one
/// of its superclasses, superinterfaces, or implemented interfaces. Access
/// control is not enforced; for example, private methods can be invoked.
///
/// The method invocation will occur in the specified thread. Method invocation
/// can occur only if the specified thread has been suspended by an event.
/// Method invocation is not supported when the target VM has been suspended by
/// the front-end.
///
/// The specified method is invoked with the arguments in the specified argument
/// list. The method invocation is synchronous; the reply packet is not sent
/// until the invoked method returns in the target VM. The return value
/// (possibly the void value) is included in the reply packet. If the invoked
/// method throws an exception, the exception object ID is set in the reply
/// packet; otherwise, the exception object ID is null.
///
/// For primitive arguments, the argument value's type must match the argument's
/// type exactly. For object arguments, there must exist a widening reference
/// conversion from the argument value's type to the argument's type and the
/// argument's type must be loaded.
///
/// By default, all threads in the target VM are resumed while the method is
/// being invoked if they were previously suspended by an event or by command.
/// This is done to prevent the deadlocks that will occur if any of the threads
/// own monitors that will be needed by the invoked method. It is possible that
/// breakpoints or other events might occur during the invocation. Note,
/// however, that this implicit resume acts exactly like the ThreadReference
/// [resume](super::thread_reference::Resume) command, so if the
/// thread's suspend count is greater than 1, it will remain in a suspended
/// state during the invocation. By default, when the invocation completes, all
/// threads in the target VM are suspended, regardless their state before the
/// invocation.
///
/// The resumption of other threads during the invoke can be prevented by
/// specifying the
/// [INVOKE_SINGLE_THREADED](crate::enums::InvokeOptions::SINGLE_THREADED) bit
/// flag in the options field; however, there is no protection against or
/// recovery from the deadlocks described above, so this option should be used
/// with great caution. Only the specified thread will be resumed (as described
/// for all threads above). Upon completion of a single threaded invoke, the
/// invoking thread will be suspended once again. Note that any threads started
/// during the single threaded invocation will not be suspended when the
/// invocation completes.
///
/// If the target VM is disconnected during the invoke (for example, through the
/// VirtualMachine [dispose](super::virtual_machine::Dispose) command) the
/// method invocation continues.

#[jdwp_command(3, 3)]
#[derive(Debug, JdwpWritable)]
pub struct InvokeMethod {
    /// The class type ID.
    class_id: ClassID,
    /// The thread in which to invoke.
    thread_id: ThreadID,
    /// The method to invoke.
    method_id: MethodID,
    /// Arguments to the method.
    arguments: Vec<Value>,
    // Invocation options
    options: InvokeOptions,
}

#[jdwp_command(3, 4)]
#[derive(Debug, JdwpWritable)]
pub struct NewInstance {
    /// The class type ID.
    class_id: ClassID,
    /// The thread in which to invoke the constructor.
    thread_id: ThreadID,
    /// The constructor to invoke.
    method_id: MethodID,
    /// Arguments for the constructor method.
    arguments: Vec<Value>,
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
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid NewInstance reply",
            )),
        }
    }
}
