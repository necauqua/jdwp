use std::fmt::Debug;

use jdwp_macros::jdwp_command;

use crate::{
    codec::{JdwpReadable, JdwpWritable},
    CommandId,
};

pub mod array_reference;
pub mod array_type;
pub mod class_loader_reference;
pub mod class_object_reference;
pub mod event;
pub mod event_request;
pub mod object_reference;
pub mod reference_type;
pub mod string_reference;
pub mod thread_group_reference;
pub mod virtual_machine;

pub mod field {
    // no commands defined in this set
}

pub mod interface_type {
    // no commands defined in this set
}

pub trait Command: JdwpWritable + Debug {
    const ID: CommandId;

    type Output: JdwpReadable + Debug;
}
