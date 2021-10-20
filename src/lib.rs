#![allow(dead_code)]

extern crate self as jdwp;

use std::io::{Read, Write};

use crate::{
    codec::{JdwpReadable, JdwpWritable},
    enums::{ErrorCode, Flags},
};

mod codec;

pub mod client;
pub mod commands;
pub mod enums;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, JdwpReadable, JdwpWritable)]
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

#[derive(Copy, Clone)]
enum PacketMeta {
    Command(CommandId),
    Reply(ErrorCode),
}

#[derive(Copy, Clone)]
pub struct PacketHeader {
    length: u32,
    id: u32,
    flags: Flags,
    meta: PacketMeta,
}

impl JdwpReadable for PacketHeader {
    fn read<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let length = u32::read(read)?;
        let id = u32::read(read)?;
        let flags = Flags::read(read)?;
        let meta = match flags {
            Flags::Command => PacketMeta::Command(CommandId::read(read)?),
            Flags::Reply => PacketMeta::Reply(ErrorCode::read(read)?),
        };
        Ok(PacketHeader {
            length,
            id,
            flags,
            meta,
        })
    }
}

impl JdwpWritable for PacketHeader {
    fn write<W: Write>(&self, write: &mut W) -> std::io::Result<()> {
        self.length.write(write)?;
        self.id.write(write)?;
        self.flags.write(write)?;
        match self.meta {
            PacketMeta::Command(id) => id.write(write),
            PacketMeta::Reply(error_code) => error_code.write(write),
        }
    }
}

pub struct Packet<'a> {
    header: &'a PacketHeader,
    data: &'a [u8],
}
