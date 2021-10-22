use crate::{
    codec::{JdwpReadable, JdwpWritable},
    commands::Command,
    Flags, PacketHeader, PacketMeta,
};
use std::{
    io::{Cursor, ErrorKind, Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

pub struct JdwpClient {
    stream: TcpStream,
    last_id: u32,
}

impl JdwpClient {
    pub fn attach<A: ToSocketAddrs>(addr: A) -> std::io::Result<JdwpClient> {
        let mut stream = TcpStream::connect(addr)?;
        let handshake = &mut [0; 14];
        *handshake = *b"JDWP-Handshake";
        stream.write_all(handshake)?;
        stream.read_exact(handshake)?;
        if handshake != b"JDWP-Handshake" {
            Err(std::io::Error::from(ErrorKind::InvalidData))
        } else {
            Ok(JdwpClient { stream, last_id: 0 })
        }
    }

    pub fn send<C: Command>(&mut self, command: C) -> std::io::Result<(PacketHeader, C::Output)> {
        let id = self.last_id + 1;
        self.last_id = id;
        let mut data = Vec::new();
        command.write(&mut data)?;
        let header = PacketHeader {
            length: 11 + data.len() as u32,
            id,
            flags: Flags::Command,
            meta: PacketMeta::Command(C::ID),
        };
        header.write(&mut self.stream)?;
        self.stream.write_all(&data)?;
        let header = PacketHeader::read(&mut self.stream)?;
        let len = header.length as usize - 11;
        let mut data = vec![0; len];
        self.stream.read_exact(&mut data)?;
        let mut cursor = Cursor::new(data);
        let result = C::Output::read(&mut cursor)?;
        // if cursor.position() < len as u64 {
        //     println!("did not read the entire packet!");
        // } else {
        //     println!("read the entire packet, good");
        // }
        Ok((header, result))
    }
}
