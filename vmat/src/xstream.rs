use crate::alphabet::Character;
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

    fn read_until(
        &mut self,
        buf: &mut [Self::CharType],
        delimiter: Self::CharType,
    ) -> Result<usize, std::io::Error>;

    /// Reads the next char from stream, if any.
    fn get(&mut self) -> Result<Option<Self::CharType>, std::io::Error>;
}

pub struct XStrStream<'a, C>
where
    C: Character,
{
    xstr: &'a XString<C>,
    cur: usize,
}

impl<'a, C> XStrStream<'a, C>
where
    C: Character,
{
    pub fn open(xstr: &'a XString<C>) -> Self {
        XStrStream { xstr, cur: 0 }
    }

    pub fn close(self) {}
}

impl<'a, C> XStream for XStrStream<'a, C>
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

    fn read_until(&mut self, buf: &mut [C], delimiter: C) -> Result<usize, std::io::Error> {
        let nitems = self.xstr[self.cur..]
            .iter()
            .position(|&x| x == delimiter)
            .unwrap_or_else(|| self.xstr.len() - self.cur);
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
        let file = File::open(path)?;
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
        let m;
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

    fn read_until(&mut self, buf: &mut [C], delimiter: C) -> Result<usize, std::io::Error> {
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
        m /= self.char_bytes;
        let del_pos = buf[..m]
            .iter()
            .position(|&x| x == delimiter)
            .unwrap_or_else(|| m);
        self.seek(SeekFrom::Current(del_pos as i64 - m as i64))?;
        Ok(del_pos)
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
        let mut stream = XStrStream::open(&xstr);
        let mut buf = [0; 3];
        let mut sum_chars = 0;
        let mut n = stream.read(&mut buf).unwrap();
        while n > 0 {
            sum_chars += n;
            n = stream.read(&mut buf).unwrap();
        }
        assert_eq!(sum_chars, s.len());
        stream.close();
        xstr.append_from_slice(s);
        sum_chars = 0;
        stream = XStrStream::open(&xstr);
        n = stream.read(&mut buf).unwrap();
        while n > 0 {
            sum_chars += n;
            n = stream.read(&mut buf).unwrap();
        }
        assert_eq!(sum_chars, 2 * s.len());
        stream.close();
    }

    #[test]
    fn test_xstrstream_read_until() {
        let xstr: XString<u8> = XString::from("abcdefghijklmnopqrstuvwxyz".as_bytes());
        let mut stream = XStrStream::open(&xstr);
        let mut buf = [0; 12];
        let n = stream.read_until(&mut buf, 'f' as u8).unwrap();
        assert_eq!(n, 5);
        let n = stream.read_until(&mut buf, 'r' as u8).unwrap();
        assert_eq!(n, 12);
        let n = stream.read_until(&mut buf, 'a' as u8).unwrap();
        assert_eq!(n, 9);
        stream.close();
    }

    const DNA_FILE: &'static [u8] = b"AAAAAAAAAACCCCCCCCCCGGGGGGGGGGTTTTTTTTTT";

    fn file_setup(filename: &str) {
        let f = File::create(filename).expect("Unabe to create the file");
        let mut writer = BufWriter::new(f);
        writer.write(&DNA_FILE);
    }

    fn file_teardown(filename: &str) {
        std::fs::remove_file(filename).expect("Unable to delete file");
    }

    #[test]
    fn test_xstrfilestream_read() {
        file_setup("test_xstrfilestream_read.txt");

        let mut stream: XStrFileReader<u8> =
            XStrFileReader::new("test_xstrfilestream_read.txt").expect("Cannot open stream");
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

        file_teardown("test_xstrfilestream_read.txt");
    }

    #[test]
    fn test_xstrfilestream_read_until() {
        file_setup("test_xstrfilestream_read_until.txt");

        let mut stream: XStrFileReader<u8> =
            XStrFileReader::new("test_xstrfilestream_read_until.txt").expect("Cannot open stream");
        let mut buf: [u8; 50] = [0; 50];
        let mut cur = 0;
        let mut m = stream
            .read_until(&mut buf, 'A' as u8)
            .expect("cannot read from stream");
        assert_eq!(m, 0);
        cur += m;
        println!("cur={0} buf={1:?}", cur, &buf[..m]);
        m = stream
            .read_until(&mut buf, 'C' as u8)
            .expect("cannot read from stream");
        assert_eq!(m, 10);
        cur += m;
        println!("cur={0} buf={1:?}", cur, &buf[..m]);
        m = stream
            .read_until(&mut buf, 'T' as u8)
            .expect("cannot read from stream");
        assert_eq!(m, 20);
        cur += m;
        println!("cur={0} buf={1:?}", cur, &buf[..m]);
        m = stream
            .read_until(&mut buf, 'T' as u8)
            .expect("cannot read from stream");
        assert_eq!(m, 0);
        cur += m;
        println!("cur={0} buf={1:?}", cur, &buf[..m]);
        m = stream
            .read_until(&mut buf, 'A' as u8)
            .expect("cannot read from stream");
        assert_eq!(m, 10);
        cur += m;
        println!("cur={0} buf={1:?}", cur, &buf[..m]);
        for j in 0..m {
            assert_eq!(buf[j], DNA_FILE[cur - m + j]);
        }

        file_teardown("test_xstrfilestream_read_until.txt");
    }
}
