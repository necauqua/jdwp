#![deny(missing_debug_implementations, clippy::undocumented_unsafe_blocks)]

extern crate self as jdwp;

use std::fmt::{Display, Formatter};

use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    enums::ErrorCode,
};

pub mod client;
pub mod codec;
pub mod commands;
pub mod enums;
pub mod event_modifier;
pub mod jvm;
pub mod types;

mod functional;
mod xorshift;

#[derive(Copy, Clone, Debug, PartialEq, Eq, JdwpReadable, JdwpWritable)]
pub struct CommandId {
    command_set: u8,
    command: u8,
}

impl CommandId {
    pub(crate) const fn new(command_set: u8, command: u8) -> CommandId {
        CommandId {
            command_set,
            command,
        }
    }
}

impl Display for CommandId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.command_set, self.command)
    }
}

#[derive(Debug, Copy, Clone, JdwpReadable, JdwpWritable)]
#[repr(u8)]
enum PacketMeta {
    Command(CommandId) = 0x00,
    Reply(ErrorCode) = 0x80,
}

#[derive(Debug, Copy, Clone, JdwpReadable, JdwpWritable)]
pub struct PacketHeader {
    length: u32,
    id: u32,
    meta: PacketMeta,
}

impl PacketHeader {
    pub(crate) const JDWP_SIZE: usize = 11;
}
