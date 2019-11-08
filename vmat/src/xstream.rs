use crate::alphabet::{Alphabet, Character};
use crate::xstring::XString;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;
use std::slice;

pub trait XStream {
    type CharType;

    /// Reads from stream into the given buffer.
    /// Returns the number of items (chars) read.
    fn read(&mut self, buf: &mut [Self::CharType]) -> Result<usize, std::io::Error>;

    /// Reads the next char from stream, if any.
    fn get(&mut self) -> Result<Option<Self::CharType>, std::io::Error>;
}

pub struct XStrStream<C>
where
    C: Character,
{
    xstr: XString<C>,
    cur: usize,
}

impl<C> XStrStream<C>
where
    C: Character,
{
    pub fn open(xstr: XString<C>) -> Self {
        XStrStream { xstr, cur: 0 }
    }

    pub fn close(self) -> XString<C> {
        self.xstr
    }
}

impl<C> XStream for XStrStream<C>
where
    C: Character,
{
    type CharType = C;

    fn get(&mut self) -> Result<Option<C>, std::io::Error> {
        if self.cur < self.xstr.len() {
            self.cur += 1;
            Ok(Some(self.xstr[self.cur - 1]))
        } else {
            Ok(None)
        }
    }

    fn read(&mut self, buf: &mut [C]) -> Result<usize, std::io::Error> {
        let nitems = std::cmp::min(buf.len(), self.xstr.len() - self.cur);
        buf[..nitems].copy_from_slice(&self.xstr[self.cur..self.cur + nitems]);
        self.cur += nitems;
        Ok(nitems)
    }
}

use std::marker::PhantomData;
use std::mem::size_of;

pub struct XStrFileReader<C> {
    any_char: PhantomData<C>,
    char_bytes: usize,
    freader: BufReader<File>,
}

impl<C> XStrFileReader<C>
where
    C: Character,
{
    pub fn new_from_file(src: File) -> Result<Self, std::io::Error> {
        Ok(XStrFileReader {
            any_char: PhantomData,
            char_bytes: size_of::<C>(),
            freader: BufReader::new(src),
        })
    }

    pub fn new<P>(path: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path)?;
        Self::new_from_file(file)
    }
}

/// Convert a slice of T (where T is plain old data) to its mutable binary
/// representation.
///
/// This function is wildly unsafe because it permits arbitrary modification of
/// the binary representation of any `Copy` type. Use with care.
unsafe fn slice_to_u8_mut<T: Copy>(slice: &mut [T]) -> &mut [u8] {
    let len = size_of::<T>() * slice.len();
    slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, len)
}

impl<C> XStream for XStrFileReader<C>
where
    C: Character,
{
    type CharType = C;
    fn get(&mut self) -> Result<Option<C>, std::io::Error> {
        let mut c: [C; 1] = [Default::default(); 1];
        if self.read(&mut c)? == 1 {
            Ok(Some(c[0]))
        } else {
            Ok(None)
        }
    }

    fn read(&mut self, buf: &mut [C]) -> Result<usize, std::io::Error> {
        let mut m;
        unsafe {
            let rawbuf =
                slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * self.char_bytes);
            m = self.freader.read(rawbuf)?;
        }
        if m % self.char_bytes != 0 {
            return Result::Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid number or bytes",
            ));
        }
        Ok(m / self.char_bytes)
    }
}

impl<C> Seek for XStrFileReader<C>
where
    C: Character,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        return self.freader.seek(pos);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::xstring::*;
    use std::io::{BufWriter, Write};

    #[test]
    fn test_xstrstream() {
        let mut xstr: XString<u8> = XString::new();
        let s = "abcdefghijklmnopqrstuvwxyz".as_bytes();
        xstr.append_from_slice(s);
        let mut stream = XStrStream::open(xstr);
        let mut buf = [0; 3];
        let mut sum_chars = 0;
        let mut n = stream.read(&mut buf).unwrap();
        while n > 0 {
            sum_chars += n;
            n = stream.read(&mut buf).unwrap();
        }
        assert_eq!(sum_chars, s.len());
        xstr = stream.close();
        xstr.append_from_slice(s);
        sum_chars = 0;
        stream = XStrStream::open(xstr);
        n = stream.read(&mut buf).unwrap();
        while n > 0 {
            sum_chars += n;
            n = stream.read(&mut buf).unwrap();
        }
        assert_eq!(sum_chars, 2 * s.len());
        stream.close();
    }

    const DNA_FILE: &'static [u8] = b"aaaaaaaaaaccccccccccggggggggggtttttttttt";

    fn file_setup() {
        let f = File::create("test.txt").expect("Unabe to create the file");
        let mut writer = BufWriter::new(f);
        writer.write(&DNA_FILE);
    }

    fn file_teardown() {
        std::fs::remove_file("test.txt").expect("Unable to delete file");
    }

    #[test]
    fn test_xstrfilestream() {
        file_setup();

        let mut stream: XStrFileReader<u8> =
            XStrFileReader::new("test.txt").expect("Cannot open stream");
        let mut i = 0;
        let mut c = stream.get().expect("cannot read from stream");
        while c.is_some() {
            assert_eq!(c.unwrap(), DNA_FILE[i]);
            c = stream.get().expect("cannot read from stream");
            i += 1;
        }
        assert_eq!(i, DNA_FILE.len());

        i = 0;
        let mut buf: [u8; 7] = [0; 7];
        stream
            .seek(SeekFrom::Start(0))
            .expect("cannot rewind stream");
        let mut m = stream.read(&mut buf).expect("cannot read from stream");
        while m > 0 {
            assert_eq!(buf[..m], DNA_FILE[i..i + m]);
            i += m;
            m = stream.read(&mut buf).expect("cannot read from stream");
        }

        file_teardown();
    }
}
