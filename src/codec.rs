use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Error, ErrorKind, Read, Write};

pub use jdwp_macros::{JdwpReadable, JdwpWritable};

pub trait JdwpReadable: Sized {
    fn read<R: Read>(read: &mut R) -> io::Result<Self>;
}

pub trait JdwpWritable {
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()>;
}

impl JdwpReadable for () {
    #[inline]
    fn read<R: Read>(_: &mut R) -> io::Result<Self> {
        Ok(())
    }
}

impl JdwpWritable for () {
    #[inline]
    fn write<W: Write>(&self, _: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl JdwpReadable for bool {
    #[inline]
    fn read<R: Read>(read: &mut R) -> io::Result<Self> {
        read.read_u8().map(|n| n != 0)
    }
}

impl JdwpWritable for bool {
    #[inline]
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write.write_u8(if *self { 1 } else { 0 })
    }
}

impl JdwpReadable for u8 {
    #[inline]
    fn read<R: Read>(read: &mut R) -> io::Result<Self> {
        read.read_u8()
    }
}

impl JdwpWritable for u8 {
    #[inline]
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write.write_u8(*self)
    }
}

impl JdwpReadable for u16 {
    #[inline]
    fn read<R: Read>(read: &mut R) -> io::Result<Self> {
        read.read_u16::<BigEndian>()
    }
}

impl JdwpWritable for u16 {
    #[inline]
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write.write_u16::<BigEndian>(*self)
    }
}

impl JdwpReadable for u32 {
    #[inline]
    fn read<R: Read>(read: &mut R) -> io::Result<Self> {
        read.read_u32::<BigEndian>()
    }
}

impl JdwpWritable for u32 {
    #[inline]
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write.write_u32::<BigEndian>(*self)
    }
}

impl JdwpReadable for u64 {
    #[inline]
    fn read<R: Read>(read: &mut R) -> io::Result<Self> {
        read.read_u64::<BigEndian>()
    }
}

impl JdwpWritable for u64 {
    #[inline]
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        write.write_u64::<BigEndian>(*self)
    }
}

impl JdwpReadable for String {
    #[inline]
    fn read<R: Read>(read: &mut R) -> io::Result<Self> {
        let mut bytes = vec![0; u32::read(read)? as usize];
        read.read_exact(&mut bytes)?;
        String::from_utf8(bytes).map_err(|_| Error::from(ErrorKind::InvalidData))
    }
}

impl JdwpWritable for String {
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        (self.len() as u32).write(write)?;
        write.write_all(self.as_bytes())
    }
}

impl<T: JdwpReadable> JdwpReadable for Vec<T> {
    fn read<R: Read>(read: &mut R) -> io::Result<Self> {
        let len = u32::read(read)?;
        let mut res = Vec::with_capacity(len as usize);
        for _ in 0..len {
            res.push(T::read(read)?);
        }
        Ok(res)
    }
}

impl<T: JdwpWritable> JdwpWritable for Vec<T> {
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
        (self.len() as u32).write(write)?;
        for item in self {
            item.write(write)?;
        }
        Ok(())
    }
}
