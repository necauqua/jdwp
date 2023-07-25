use crate::{
    codec::{JdwpReadable, JdwpWritable},
    spec::ErrorCode,
};
use std::{fmt, fmt::Display};

pub trait Command {
    const ID: CommandId;

    type Output;
}

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.command_set, self.command)
    }
}

#[derive(Debug, Copy, Clone, JdwpReadable, JdwpWritable)]
#[repr(u8)]
pub enum PacketMeta {
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
    pub const fn new(length: u32, id: u32, meta: PacketMeta) -> PacketHeader {
        PacketHeader { length, id, meta }
    }

    pub const fn length(&self) -> u32 {
        self.length
    }

    pub const fn id(&self) -> u32 {
        self.id
    }

    pub const fn meta(&self) -> PacketMeta {
        self.meta
    }
}

impl PacketHeader {
    pub(crate) const JDWP_SIZE: u32 = 4 + 4 + 1 + 2;
}
