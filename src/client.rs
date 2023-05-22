use std::{
    collections::HashMap,
    io::{self, Cursor, Read, Write},
    net::{Shutdown, TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use thiserror::Error;

use crate::{
    codec::{JdwpReadable, JdwpReader, JdwpWritable, JdwpWriter},
    commands::{
        event::Composite,
        virtual_machine::{Dispose, IDSizeInfo},
        Command,
    },
    xorshift::XorShift32,
    ErrorCode, PacketHeader, PacketMeta,
};

type WaitingMap = Arc<Mutex<HashMap<u32, Sender<Result<Vec<u8>, ClientError>>>>>;

#[derive(Debug)]
pub struct JdwpClient {
    writer: JdwpWriter<TcpStream>,
    host_events_rx: Receiver<Composite>,
    waiting: WaitingMap,
    next_id: XorShift32,
    reader_handle: Option<JoinHandle<ClientError>>,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Failed handshake")]
    FailedHandshake,
    #[error("{0}")]
    HostError(ErrorCode),
    #[error("Too much data received from the host ({actual}/{expected} bytes)")]
    TooMuchDataReceived { expected: usize, actual: usize },
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("The client was disposed either by the Dispose command or by a network error already consumed")]
    Disposed,
}

const HANDSHAKE: &[u8] = b"JDWP-Handshake";

impl JdwpClient {
    pub fn attach<A: ToSocketAddrs>(addr: A) -> Result<JdwpClient, ClientError> {
        let mut stream = TcpStream::connect(addr)?;

        stream.write_all(HANDSHAKE)?;
        let handshake = &mut [0; HANDSHAKE.len()];
        stream.read_exact(handshake)?;
        if handshake != HANDSHAKE {
            return Err(ClientError::FailedHandshake);
        }

        let waiting = Arc::new(Mutex::new(HashMap::new()));
        let (host_events_tx, host_events_rx) = mpsc::channel();

        // todo: hardcode fetching it here I guess
        let id_sizes = IDSizeInfo {
            field_id_size: 8,
            method_id_size: 8,
            object_id_size: 8,
            reference_type_id_size: 8,
            frame_id_size: 8,
        };

        let reader_handle = thread::spawn({
            let mut reader = JdwpReader::new(stream.try_clone()?, id_sizes.clone());
            let waiting = waiting.clone();
            move || loop {
                if let Err(e) = read_packet(&mut reader, &waiting, &host_events_tx) {
                    log::error!("Failed to read incoming data: {}", e);
                    break e;
                }
            }
        });

        Ok(JdwpClient {
            writer: JdwpWriter::new(stream, id_sizes),
            host_events_rx,
            waiting,
            next_id: XorShift32::new(0xDEAD),
            reader_handle: Some(reader_handle),
        })
    }

    pub fn host_events(&self) -> &Receiver<Composite> {
        &self.host_events_rx
    }

    pub fn send<C: Command>(&mut self, command: C) -> Result<C::Output, ClientError> {
        match self.reader_handle {
            Some(ref handle) if handle.is_finished() => {
                return Err(self.reader_handle.take().unwrap().join().unwrap())
            }
            None => return Err(ClientError::Disposed),
            _ => {}
        }

        let (waiting_tx, waiting_rx) = mpsc::channel();

        let id = self.next_id.next();

        // see comment below
        if C::ID != Dispose::ID {
            self.waiting.lock().unwrap().insert(id, waiting_tx);
        }

        let mut data = Vec::new();
        command.write(&mut JdwpWriter::new(
            &mut data,
            self.writer.id_sizes.clone(),
        ))?;

        let header = PacketHeader {
            length: (PacketHeader::JDWP_SIZE + data.len()) as u32,
            id,
            meta: PacketMeta::Command(C::ID),
        };

        header.write(&mut self.writer)?;
        self.writer.write_all(&data)?;

        log::trace!("[{:x}] sent command {}: {:?}", header.id, C::ID, command);

        // special handling for the dispose command because
        // we don't always get the response header for it
        if C::ID == Dispose::ID {
            // stop the reading thread by closing the socket
            self.writer.shutdown(Shutdown::Both)?;

            // force next calls to send to return ClientError::Disposed,
            // otherwise we get either UnexpectedEof or BrokenPipe and maybe
            // something else from closing the socket
            self.reader_handle = None;

            // SAFETY: we know that C is () here, but the type system does not, eh
            // technically it's a noop, we just cheat the types
            // can do this in safe Rust with trait specialization whenever that's in the
            // language

            // todo: now years later I'm not too sure about this?.. it's fishy
            return Ok(unsafe { std::mem::transmute_copy(&()) });
        }

        let data = waiting_rx
            .recv()
            .expect("Sender hung up, this cannot happen")?;

        let len = data.len();
        let mut cursor = Cursor::new(data);
        let result = C::Output::read(&mut JdwpReader::new(
            &mut cursor,
            self.writer.id_sizes.clone(),
        ))?;

        log::trace!("[{:x}] data: {:#?}", header.id, result);

        if cursor.position() < len as u64 {
            Err(ClientError::TooMuchDataReceived {
                actual: len,
                expected: cursor.position() as usize,
            })
        } else {
            Ok(result)
        }
    }
}

fn read_packet(
    reader: &mut JdwpReader<TcpStream>,
    waiting: &WaitingMap,
    host_events_tx: &Sender<Composite>,
) -> Result<(), ClientError> {
    let header = PacketHeader::read(reader)?;
    let mut data = vec![0; header.length as usize - PacketHeader::JDWP_SIZE];

    reader.read_exact(&mut data)?;

    let to_send = match header.meta {
        // handle the host-sent commands;
        // the only one is the Event command
        PacketMeta::Command(Composite::ID) => {
            let composite = Composite::read(&mut JdwpReader::new(
                &mut Cursor::new(data),
                reader.id_sizes.clone(),
            ))?;

            log::trace!("[host] event: {:#?}", composite);

            host_events_tx.send(composite).unwrap();
            return Ok(());
        }
        PacketMeta::Command(command_id) => {
            log::warn!(
                "Unknown command received from the host, ignoring: {}",
                command_id
            );
            return Ok(());
        }
        PacketMeta::Reply(ErrorCode::None) => {
            log::trace!("[{:x}] reply, len {}", header.id, data.len());
            Ok(data)
        }
        PacketMeta::Reply(error_code) => {
            log::trace!("[{:x}] reply, host error: {:?}", header.id, error_code);
            Err(ClientError::HostError(error_code))
        }
    };

    match waiting.lock().unwrap().remove(&header.id) {
        Some(waiter) => waiter.send(to_send).unwrap(), // one-shot channel send
        None => log::warn!(
            "Received an unexpected packet from the JVM, ignoring: {:?}",
            header
        ),
    }
    Ok(())
}
