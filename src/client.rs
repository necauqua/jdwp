use std::{
    collections::HashMap,
    fmt::Debug,
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
    spec::{
        event::Composite,
        virtual_machine::{Dispose, IDSizeInfo, IDSizes},
        Command, ErrorCode, PacketHeader, PacketMeta,
    },
    xorshift::XorShift32,
};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Failed handshake")]
    FailedHandshake,
    #[error("Failed initial IDSizes call")]
    FailedInitialIdSizesCall(Option<ErrorCode>),
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

#[derive(Debug)]
pub struct JdwpClient {
    stream: TcpStream,
    id_sizes: IDSizeInfo,
    next_id: XorShift32,
    shared: Arc<Mutex<SharedData>>,
    reader_handle: Option<JoinHandle<ClientError>>,
}

#[derive(Debug)]
struct SharedData {
    waiting: HashMap<u32, Sender<Result<Vec<u8>, ClientError>>>,
    event_senders: Vec<Sender<Composite>>,
}

impl Drop for JdwpClient {
    fn drop(&mut self) {
        _ = self.stream.shutdown(Shutdown::Read);
    }
}

impl JdwpClient {
    pub fn attach(mut stream: TcpStream) -> Result<Self, ClientError> {
        stream.write_all(HANDSHAKE)?;
        let handshake = &mut [0; HANDSHAKE.len()];
        stream.read_exact(handshake)?;
        if handshake != HANDSHAKE {
            return Err(ClientError::FailedHandshake);
        }

        // a stub id_sizes just to have something in the bootstrap writer and reader
        // it's fine to have whatever as we are not reading any object-ids anyway
        let id_sizes = IDSizeInfo::default();

        // manually send an IDSizes command
        let mut bootstrap_writer = JdwpWriter::new(&mut stream, &id_sizes);
        let id_sizes_header =
            PacketHeader::new(PacketHeader::JDWP_SIZE, 1, PacketMeta::Command(IDSizes::ID));
        id_sizes_header.write(&mut bootstrap_writer)?;

        // and then manually read the actual sizes from the response
        let mut bootstrap_reader = JdwpReader::new(&mut stream, &id_sizes);
        let reply_header = PacketHeader::read(&mut bootstrap_reader)?;
        if reply_header.id() != 1 || reply_header.length() != PacketHeader::JDWP_SIZE + 20 {
            return Err(ClientError::FailedInitialIdSizesCall(None));
        }
        let id_sizes = match reply_header.meta() {
            PacketMeta::Reply(ErrorCode::None) => IDSizeInfo::read(&mut bootstrap_reader)?,
            PacketMeta::Command(_) => return Err(ClientError::FailedInitialIdSizesCall(None)),
            PacketMeta::Reply(error_code) => {
                return Err(ClientError::FailedInitialIdSizesCall(Some(error_code)))
            }
        };

        log::debug!("Received IDSizes reply: {id_sizes:?}");

        let shared = Arc::new(Mutex::new(SharedData {
            waiting: HashMap::new(),
            event_senders: Vec::new(),
        }));

        let reader_handle = thread::spawn({
            let id_sizes = id_sizes.clone();
            let stream = stream.try_clone()?;
            let shared = shared.clone();
            move || {
                let mut reader = JdwpReader::new(stream, &id_sizes);
                loop {
                    if let Err(e) = read_packet(&mut reader, &shared) {
                        log::error!("Failed to read incoming data: {e}");
                        break e;
                    }
                }
            }
        });

        Ok(Self {
            stream,
            id_sizes,
            next_id: XorShift32::new(0xDEAD),
            shared,
            reader_handle: Some(reader_handle),
        })
    }

    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, ClientError> {
        Self::attach(TcpStream::connect(addr)?)
    }

    pub fn receive_events(&self) -> Receiver<Composite> {
        let (tx, rx) = mpsc::channel();
        self.shared.lock().unwrap().event_senders.push(tx);
        rx
    }

    pub fn send<C>(&mut self, command: C) -> Result<C::Output, ClientError>
    where
        C: Command + JdwpWritable + Debug,
        C::Output: JdwpReadable + Debug,
    {
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
            self.shared.lock().unwrap().waiting.insert(id, waiting_tx);
        }

        // the capacity is a random heuristic, should probably look into adjusting this
        // when a bunch of tests are written
        let mut data = Vec::with_capacity(256);

        // write the command separately to learn the payload length
        command.write(&mut JdwpWriter::new(&mut data, &self.id_sizes))?;

        let len = PacketHeader::JDWP_SIZE + data.len() as u32;
        let header = PacketHeader::new(len, id, PacketMeta::Command(C::ID));

        // and then make another writer to use the JdwpWritable derive for the header
        // ¯\_(ツ)_/¯
        header.write(&mut JdwpWriter::new(&mut self.stream, &self.id_sizes))?;
        self.stream.write_all(&data)?;

        log::trace!("[{:x}] sent command {}: {command:?}", header.id(), C::ID);

        // special handling for the dispose command because
        // we don't always get the response header for it
        if C::ID == Dispose::ID {
            // stop the reading thread by closing the socket
            self.stream.shutdown(Shutdown::Both)?;

            // force next calls to send to return ClientError::Disposed,
            // otherwise we get either UnexpectedEof or BrokenPipe or maybe
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

        let result = C::Output::read(&mut JdwpReader::new(&mut cursor, &self.id_sizes))?;

        log::trace!("[{:x}] data: {:#?}", header.id(), result);

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
    shared: &Arc<Mutex<SharedData>>,
) -> Result<(), ClientError> {
    let header = PacketHeader::read(reader)?;

    let mut data = vec![0; (header.length() - PacketHeader::JDWP_SIZE) as usize];
    reader.read_exact(&mut data)?;

    let to_send = match header.meta() {
        // handle the host-sent commands
        // the only one is the Composite Event command
        PacketMeta::Command(Composite::ID) => {
            let mut cursor = Cursor::new(data);
            let mut reader = JdwpReader::new(&mut cursor, reader.id_sizes);
            let composite = Composite::read(&mut reader)?;

            log::trace!("[host] event: {:#?}", composite);

            let mut shared = shared.lock().unwrap();

            // ugh it's pretty ugly to avoid one extra clone (and extra clones when
            // receivers were dropped) and I haven't even made the retain part
            // work it will optimize?.
            shared
                .event_senders
                .retain(|sender| sender.send(composite.clone()).is_ok());

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
            log::trace!("[{:x}] reply, len {}", header.id(), data.len());
            Ok(data)
        }
        PacketMeta::Reply(error_code) => {
            log::trace!("[{:x}] reply, host error: {:?}", header.id(), error_code);
            Err(ClientError::HostError(error_code))
        }
    };

    match shared.lock().unwrap().waiting.remove(&header.id()) {
        Some(waiter) => waiter.send(to_send).unwrap(), // one-shot channel send
        None => log::warn!(
            "Received an unexpected packet from the JVM, ignoring: {:?}",
            header
        ),
    }
    Ok(())
}
