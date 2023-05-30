use jdwp_macros::{jdwp_command, JdwpWritable};

use crate::{
    enums::InvokeOptions,
    types::{InterfaceID, InvokeMethodReply, MethodID, ThreadID, Value},
};

/// Invokes a static method. The method must not be a static initializer.
/// The method must be a member of the interface type.
///
/// Since JDWP version 1.8
///
/// The method invocation will occur in the specified thread. Method
/// invocation can occur only if the specified thread has been suspended by
/// an event. Method invocation is not supported when the target VM has been
/// suspended by the front-end.
///
/// The specified method is invoked with the arguments in the specified
/// argument list. The method invocation is synchronous; the reply packet is
/// not sent until the invoked method returns in the target VM. The return
/// value (possibly the void value) is included in the reply packet. If the
/// invoked method throws an exception, the exception object ID is set in
/// the reply packet; otherwise, the exception object ID is null.
///
/// For primitive arguments, the argument value's type must match the
/// argument's type exactly. For object arguments, there must exist a
/// widening reference conversion from the argument value's type to the
/// argument's type and the argument's type must be loaded.
///
/// By default, all threads in the target VM are resumed while the method is
/// being invoked if they were previously suspended by an event or by a
/// command. This is done to prevent the deadlocks that will occur if any of
/// the threads own monitors that will be needed by the invoked method. It
/// is possible that breakpoints or other events might occur during the
/// invocation. Note, however, that this implicit resume acts exactly like
/// the ThreadReference resume command, so if the thread's suspend count is
/// greater than 1, it will remain in a suspended state during the
/// invocation. By default, when the invocation completes, all threads in
/// the target VM are suspended, regardless their state before the
/// invocation.
///
/// The resumption of other threads during the invoke can be prevented by
/// specifying the SINGLE_THREADED bit flag in the options field;
/// however, there is no protection against or recovery from the deadlocks
/// described above, so this option should be used with great caution. Only
/// the specified thread will be resumed (as described for all threads
/// above). Upon completion of a single threaded invoke, the invoking thread
/// will be suspended once again. Note that any threads started during the
/// single threaded invocation will not be suspended when the invocation
/// completes.

// If the target VM is disconnected during the invoke (for example, through the VirtualMachine
// [Dispose](super::virtual_machine::Dispose) command) the method invocation continues.
#[jdwp_command(5, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct InvokeMethod<'a> {
    /// The interface type ID
    interface_id: InterfaceID,
    /// The thread in which to invoke
    thread_id: ThreadID,
    /// The method to invoke
    method_id: MethodID,
    /// The argument values
    arguments: &'a [Value],
    /// Invocation options
    options: InvokeOptions,
}
