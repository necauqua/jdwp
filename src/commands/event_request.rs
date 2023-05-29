use super::jdwp_command;

use crate::{
    codec::JdwpWritable,
    enums::{EventKind, SuspendPolicy},
    types::{Modifier, RequestID},
};

/// Set an event request.
///
/// When the event described by this request occurs, an event is sent from the
/// target VM.
///
/// If an event occurs that has not been requested then it is not sent from the
/// target VM.
///
/// The two exceptions to this are the VM Start Event and the VM Death Event
/// which are automatically generated events - see
/// [Composite](super::event::Composite) command for further details.
#[jdwp_command(RequestID, 15, 1)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Set {
    /// Event kind to request. Some events may require a capability in order to
    /// be requested.
    event_kind: EventKind,
    /// What threads are suspended when this event occurs?
    ///
    /// Note that the order of events and command replies accurately reflects
    /// the order in which threads are suspended and resumed.
    ///
    /// For example, if a VM-wide resume is processed before an event occurs
    /// which suspends the VM, the reply to the resume command will be written
    /// to the transport before the suspending event.
    suspend_policy: SuspendPolicy,
    /// Constraints used to control the number of generated events.
    ///
    /// Modifiers specify additional tests that an event must satisfy before it
    /// is placed in the event queue.
    ///
    /// Events are filtered by applying each modifier to an event in the order
    /// they are specified in this collection Only events that satisfy all
    /// modifiers are reported.
    ///
    /// An empty list means there are no modifiers in the request.
    ///
    /// Filtering can improve debugger performance dramatically by reducing the
    /// amount of event traffic sent from the target VM to the debugger VM.
    modifiers: Vec<Modifier>,
}

/// Clear an event request.
///
/// See [EventKind] for a complete list of events that can be cleared.
///
/// Only the event request matching the specified event kind and `request_id`
/// is cleared.
///
/// If there isn't a matching event request the command is a no-op and does not
/// result in an error.
///
/// Automatically generated events do not have a corresponding event request
/// and may not be cleared using this command.
#[jdwp_command((), 15, 2)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct Clear {
    /// Event kind to clear
    event_kind: EventKind,
    /// ID of request to clear
    request_id: RequestID,
}

/// Removes all set breakpoints, a no-op if there are no breakpoints set.
#[jdwp_command((), 15, 3)]
#[derive(Debug, Clone, JdwpWritable)]
pub struct ClearAllBreakpoints;
