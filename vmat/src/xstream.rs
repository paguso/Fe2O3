use std::io::BufRead;
use std::io::Read;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::slice::SliceIndex;
use std::ops::Index;

pub struct XStream<R> 
where R: Read + Seek
{
    reader: R,
    buf: Vec<u8>,
    bfrom: usize,
    bto: usize,
    bstart: usize,
    fpos: u64,
}

impl<R> XStream<R> 
where R: Read + Seek
{
    fn new(reader: R) -> Self {
        XStream {
            reader: reader,
            buf: Vec::new(),
            bfrom: 0,
            bto: 0,
            bstart: 0,
            fpos: 0
        }
    }

    fn read_to_buf(&mut self, offset:u64, nbytes: usize) -> Result<usize, std::io::Error> {
        self.fpos = self.reader.seek(SeekFrom::Start(offset))?;
        if nbytes > self.buf.len() {
            self.buf.resize(nbytes, 0);
        }
        let n = self.reader.read(&mut self.buf[..nbytes])?;
        self.bfrom = self.fpos as usize;
        self.bto = self.bfrom + n;
        Ok(n)
    }

}

impl<I, R> Index<I> for XStream<R>
where
    I: SliceIndex<[u8]> + std::any::Any,
    R: Read + Seek
{
    type Output = <I as SliceIndex<[u8]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        let bi = Box::new(index);
        if let Ok(i) = bi.downcast::<usize>() { 
            self.buf[i]
        }
        panic!("Invalid index"); 
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::BufWriter;
    use std::io::Read;
    use std::io::Write;

    fn test_setup() -> std::io::Result<()> {
        let mut v:Vec<u8> = vec![0];
        for i in 0..=255 {
            v.push(i);
        }
        let mut w = BufWriter::new(File::create("test.txt")?);
        for i in 0..=255 {
            v[i] = i as u8;
            w.write_all(&v)?;
        }
        w.flush()?;
        Ok(()) 
    }

    #[test]
    fn test_read() -> std::io::Result<()> {
        let s = XStream::new(BufReader::new(File::open("test.txt")?));

        Ok(())
    }
}