use std::{
    fmt::Debug,
    io::{self, Error, ErrorKind, Read, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;

pub use jdwp_macros::{JdwpReadable, JdwpWritable};

use crate::{commands::virtual_machine::IDSizeInfo, functional::Coll};

#[derive(Debug)]
pub struct JdwpWriter<W: Write> {
    write: W,
    pub(crate) id_sizes: IDSizeInfo,
}

impl<W: Write> JdwpWriter<W> {
    pub(crate) fn new(write: W, id_sizes: IDSizeInfo) -> Self {
        Self { write, id_sizes }
    }
}

impl<W: Write> Deref for JdwpWriter<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

impl<W: Write> DerefMut for JdwpWriter<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.write
    }
}

#[derive(Debug)]
pub struct JdwpReader<R: Read> {
    read: R,
    buffered_byte: Option<u8>,
    pub(crate) id_sizes: IDSizeInfo,
}

impl<R: Read> JdwpReader<R> {
    pub(crate) fn new(read: R, id_sizes: IDSizeInfo) -> Self {
        Self {
            read,
            buffered_byte: None,
            id_sizes,
        }
    }

    pub(crate) fn peek_tag_byte(&mut self) -> io::Result<u8> {
        let b = self.read.read_u8()?;
        let prev = self.buffered_byte.replace(b);
        assert!(prev.is_none(), "Already contained unconsumed tag byte");
        Ok(b)
    }

    /// This exists to first consume the peeked byte after calling
    /// [peek_tag_byte].
    ///
    /// Other read methods do not consume the buffered byte, but peek_tag_byte
    /// is only called before reading a u8 tag.
    pub(crate) fn read_tag_byte(&mut self) -> io::Result<u8> {
        match self.buffered_byte.take() {
            Some(b) => Ok(b),
            None => self.read.read_u8(),
        }
    }
}

impl<R: Read> Deref for JdwpReader<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.read
    }
}

impl<R: Read> DerefMut for JdwpReader<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.read
    }
}

pub trait JdwpReadable: Sized {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self>;
}

pub trait JdwpWritable {
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()>;
}

impl JdwpReadable for () {
    #[inline]
    fn read<R: Read>(_: &mut JdwpReader<R>) -> io::Result<Self> {
        Ok(())
    }
}

impl JdwpWritable for () {
    #[inline]
    fn write<W: Write>(&self, _: &mut JdwpWriter<W>) -> io::Result<()> {
        Ok(())
    }
}

impl<T> JdwpReadable for PhantomData<T> {
    #[inline]
    fn read<R: Read>(_: &mut JdwpReader<R>) -> io::Result<Self> {
        Ok(PhantomData)
    }
}

impl<T> JdwpWritable for PhantomData<T> {
    #[inline]
    fn write<W: Write>(&self, _: &mut JdwpWriter<W>) -> io::Result<()> {
        Ok(())
    }
}

impl JdwpReadable for bool {
    #[inline]
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        read.read_tag_byte().map(|n| n != 0)
    }
}

impl JdwpWritable for bool {
    #[inline]
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        write.write_u8(u8::from(*self))
    }
}

// read/write + i8/u8 methods do not have the endianness generic, eh

impl JdwpReadable for i8 {
    #[inline]
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        read.read_i8()
    }
}

impl JdwpWritable for i8 {
    #[inline]
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        write.write_i8(*self)
    }
}

impl JdwpReadable for u8 {
    #[inline]
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        read.read_tag_byte()
    }
}

impl JdwpWritable for u8 {
    #[inline]
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        write.write_u8(*self)
    }
}

macro_rules! int_io {
    ($($types:ident),* $(,)?) => {
        $(
            impl JdwpReadable for $types {
                #[inline]
                fn read<R: Read>(reader: &mut JdwpReader<R>) -> io::Result<Self> {
                    paste! {
                        reader.[<read_ $types>]::<BigEndian>()
                    }
                }
            }

            impl JdwpWritable for $types {
                #[inline]
                fn write<W: Write>(&self, writer: &mut JdwpWriter<W>) -> io::Result<()> {
                    paste! {
                        writer.[<write_ $types>]::<BigEndian>(*self)
                    }
                }
            }
        )*
    };
}

int_io![i16, u16, i32, u32, i64, u64, f32, f64];

impl JdwpReadable for String {
    #[inline]
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        let mut bytes = vec![0; u32::read(read)? as usize];
        read.read_exact(&mut bytes)?;
        String::from_utf8(bytes).map_err(|_| Error::from(ErrorKind::InvalidData))
    }
}

impl JdwpWritable for String {
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        (self.len() as u32).write(write)?;
        write.write_all(self.as_bytes())
    }
}

impl<C> JdwpReadable for C
where
    C: Coll + TryFrom<Vec<C::Item>>,
    C::Item: JdwpReadable,
{
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        let len = u32::read(read)?;
        if let Some(static_size) = C::STATIC_SIZE {
            if len != static_size.get() as u32 {
                return Err(Error::from(ErrorKind::InvalidData));
            }
        }
        let mut res = Vec::with_capacity(len as usize);
        for _ in 0..len {
            res.push(C::Item::read(read)?);
        }
        // SAFETY: we just checked above, so this unwrap must be a no-op
        // `Coll` is a crate-private trait only implemented for types for which that
        // check is sufficient
        Ok(unsafe { res.try_into().unwrap_unchecked() })
    }
}

impl<C> JdwpWritable for C
where
    C: Coll,
    C::Item: JdwpWritable,
{
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        (self.size() as u32).write(write)?;
        for item in self.iter() {
            item.write(write)?;
        }
        Ok(())
    }
}

impl<A: JdwpReadable, B: JdwpReadable> JdwpReadable for (A, B) {
    fn read<R: Read>(read: &mut JdwpReader<R>) -> io::Result<Self> {
        Ok((A::read(read)?, B::read(read)?))
    }
}

impl<A: JdwpWritable, B: JdwpWritable> JdwpWritable for (A, B) {
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        self.0.write(write)?;
        self.1.write(write)
    }
}

// only writable to allow using slices as command arguments
impl<T: JdwpWritable> JdwpWritable for &[T] {
    fn write<W: Write>(&self, write: &mut JdwpWriter<W>) -> io::Result<()> {
        (self.len() as u32).write(write)?;
        for item in *self {
            item.write(write)?;
        }
        Ok(())
    }
}
