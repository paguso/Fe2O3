use crate::alphabet::{Alphabet, Character};
use crate::xstring::XString;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;

pub trait XStream {
    type CharType;

    /// Returns whether the End OF Stream is reached
    fn eos(&self) -> Result<bool, std::io::Error>;

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

    fn eos(&self) -> Result<bool, std::io::Error> {
        Ok(self.cur >= self.xstr.len())
    }
}


pub struct XStrFileReader<C> {
    freader: BufReader<File>,
}

impl<C> XStrFileReader<C> 
where C: Character
{
    fn new_from_file(src: File) -> Result<Self, std::io::Error> {
        Ok( XStrFileReader {
                freader: BufReader::new(src),
            }
        )
    }

    fn new<P: AsRef<Path>> (path: P) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        Self::new_from_file(file)
    } 
}


impl<C> XStream for XStrFileReader<C> 
where C: Character,
{
    type CharType = C;
    fn get(&mut self) -> Result<Option<C>, io::Error> {

    }

    fn read(&mut self, buf: &mut [C]) -> Result<usize, io::Error> {
    }

    fn eos(&self) -> Result<bool, io::Error> {
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
