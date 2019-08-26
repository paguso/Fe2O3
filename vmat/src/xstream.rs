use std::io::BufRead;
use std::io::Read;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::slice::SliceIndex;
use std::ops::Index;
use std::default::Default;

pub struct XStream<R, C> 
where R: Read + Seek
{
    reader: R,
    buf: Vec<C>,
    fpos: u64, 
    typesize: usize
}

impl<R, C> XStream<R, C> 
where R: Read + Seek, 
      C: Default
{
    fn new(reader: R) -> Self {
        XStream {
            reader: reader,
            buf: Vec::new(),
            fpos: 0, 
            typesize: std::mem::size_of::<C>()
        }
    }

    fn read_to_buf(&mut self, offset:u64, nitems: usize) -> Result<usize, std::io::Error> {
        self.fpos = self.reader.seek(SeekFrom::Start(offset))?;
        let nbytes = self.typesize * nitems;
        if nitems > self.buf.capacity() {
            self.buf.reserve(nitems-self.buf.capacity());
        }
        let mut items_read:usize = 0;
        unsafe {
            let mut bptr = self.buf.as_mut_ptr() as *mut u8;
            let mut bufslice = std::slice::from_raw_parts_mut(bptr, nbytes) as &mut [u8];
            assert_eq!(bufslice.len(), nbytes);
            match self.reader.read(bufslice) {
                Ok(bytes_read) => {
                    items_read = bytes_read/self.typesize;
                } 
                _ => {
                    items_read = 0;
                }
            }
            self.buf.set_len(items_read);
        }
        Ok(items_read)
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
        let mut v:Vec<u16> = vec![0];
        for i in 0..1024 {
            v.push(i);
        }
        let mut w = BufWriter::new(File::create("test.txt")?);
        for i in 0..1024 {
            v[i] = i as u16;
            let buf = &v[i..i+1] as &[u8];
            w.write_all();
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