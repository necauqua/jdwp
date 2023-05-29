use jdwp_macros::jdwp_command;

use crate::{
    codec::{JdwpReadable, JdwpWritable},
    CommandId,
};

pub mod array_reference;
pub mod array_type;
pub mod class_loader_reference;
pub mod class_object_reference;
pub mod class_type;
pub mod event;
pub mod event_request;
pub mod interface_type;
pub mod method;
pub mod object_reference;
pub mod reference_type;
pub mod stack_frame;
pub mod string_reference;
pub mod thread_group_reference;
pub mod thread_reference;
pub mod virtual_machine;

/// This module is defined to mirror the JDWP command set, which is empty
pub mod field {}

pub trait Command {
    const ID: CommandId;

    type Output;
}
