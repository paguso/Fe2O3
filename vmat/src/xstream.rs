use crate::alphabet::{Alphabet, Character};
use crate::xstring::XString;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Error, ErrorKind};
use std::path::Path;

pub trait XStream {
    type CharType;

    /// Returns whether the End OF Stream is reached
    //fn eos(&self) -> Result<bool, std::io::Error>;

    /// Reads from stream into the given buffer.
    /// Returns the number of items (chars) read.
    fn read(&mut self, buf: &mut [Self::CharType]) -> Result<usize, std::io::Error>;

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

    //fn eos(&self) -> Result<bool, std::io::Error> {
    //   Ok(self.cur >= self.xstr.len())
    //}
}


pub struct XStrFileReader<C> {
    freader: BufReader<File>,
    achar: C, 
    char_bytes: usize
}

impl<C> XStrFileReader<C> 
where C: Character
{
    fn new_from_file(src: File, achar: C) -> Result<Self, std::io::Error> {
        let mut cb;
        cb = std::mem::size_of::<C>();
        Ok( XStrFileReader {
                freader: BufReader::new(src),
                achar: achar,
                char_bytes: cb
            }
        )
    }

    fn new<P: AsRef<Path>> (path: P, achar: C) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        Self::new_from_file(file, achar)
    } 
}

/// Convert a slice of T (where T is plain old data) to its mutable binary
/// representation.
///
/// This function is wildly unsafe because it permits arbitrary modification of
/// the binary representation of any `Copy` type. Use with care.
unsafe fn slice_to_u8_mut<T: Copy>(slice: &mut [T]) -> &mut [u8] {
    use std::mem::size_of;
    let len = size_of::<T>() * slice.len();
    std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, len)
}


impl<C> XStream for XStrFileReader<C> 
where C: Character,
{
    type CharType = C;
    fn get(&mut self) -> Result<Option<C>, std::io::Error> {
        let mut c: &[C] = &[Default::default()]; 
        self.read(&mut c)?;
        Ok(Some(c[0]))
    }

    fn read(&mut self, buf: &mut [C]) -> Result<usize, std::io::Error> {
        let mut m;
        unsafe {
            let mut rawbuf = std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * self.char_bytes);
            m = self.freader.read(rawbuf)?;
        }
        if m % self.char_bytes != 0 {
            return Result::Err(Error::new(ErrorKind::InvalidData, "Invalid number or bytes"));
        }
        Ok( m / self.char_bytes)
    }

}



#[cfg(test)]
mod tests {

    use super::*;
    use crate::xstring::*;

    #[test]
    fn test_xstrstream() {
        let mut xstr: XString<u8> = XString::new();
        let s = "abcdefghijklmnopqrstuvwxyz".as_bytes();
        xstr.append_from_slice(s);
        let mut stream = XStrStream::open(xstr);
        let mut buf = [0; 3];
        let mut sum_chars = 0;
        while !stream.eos().unwrap() {
            sum_chars += stream.read(&mut buf).unwrap();
        }
        assert_eq!(sum_chars, s.len());
        xstr = stream.close();
        xstr.append_from_slice(s);
        sum_chars = 0;
        stream = XStrStream::open(xstr);
        while !stream.eos().unwrap() {
            sum_chars += stream.read(&mut buf).unwrap();
        }
        assert_eq!(sum_chars, 2 * s.len());
        stream.close();
    }
}
