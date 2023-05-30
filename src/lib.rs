#![deny(missing_debug_implementations, clippy::undocumented_unsafe_blocks)]

extern crate self as jdwp;

use std::{
    fmt::{Display, Formatter},
    io::{Error, ErrorKind, Read, Write},
};

use byteorder::WriteBytesExt;

use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    enums::ErrorCode,
};

pub mod client;
pub mod codec;
pub mod commands;
pub mod enums;
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

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum PacketMeta {
    Command(CommandId) = 0x00,
    Reply(ErrorCode) = 0x80,
}

#[derive(Debug, Copy, Clone)]
pub struct PacketHeader {
    length: u32,
    id: u32,
    meta: PacketMeta,
}

impl PacketHeader {
    pub(crate) const JDWP_SIZE: usize = 11;
}

impl JdwpReadable for PacketHeader {
    fn read<R: Read>(reader: &mut JdwpReader<R>) -> std::io::Result<Self> {
        let length = u32::read(reader)?;
        let id = u32::read(reader)?;
        let meta = match u8::read(reader)? {
            0x00 => PacketMeta::Command(CommandId::read(reader)?),
            0x80 => PacketMeta::Reply(ErrorCode::read(reader)?),
            _ => Err(Error::from(ErrorKind::InvalidData))?,
        };
        Ok(PacketHeader { length, id, meta })
    }
}

impl JdwpWritable for PacketHeader {
    fn write<W: Write>(&self, writer: &mut JdwpWriter<W>) -> std::io::Result<()> {
        self.length.write(writer)?;
        self.id.write(writer)?;
        match self.meta {
            PacketMeta::Command(id) => {
                // oh well maybe someday we'll be able to get the enum discriminant
                // (or I make the derive work for such enums)
                writer.write_u8(0x00)?;
                id.write(writer)
            }
            PacketMeta::Reply(error_code) => {
                writer.write_u8(0x80)?;
                error_code.write(writer)
            }
        }
    }
}
