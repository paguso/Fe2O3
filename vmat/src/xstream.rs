use std::io::BufRead;
use std::io::Read;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::slice::SliceIndex;
use std::ops::{Index, IndexMut};
use std::default::Default;
use std::ops::Range;
use std::cell::RefCell;

struct XStreamCore<R,C>
where R: Read+Seek 
{
    reader: R,
    buf: Vec<C>,
    fpos: u64, 
    typesize: usize
}

impl<R,C> XStreamCore<R,C> 
where R: Read+Seek 
{
    fn new(reader: R, typesize: usize) -> XStreamCore<R,C> {
        XStreamCore {
            reader: reader, 
            buf: Vec::new(),
            typesize: typesize,
            fpos: 0
        }
    }
    
    fn read_to_buf(&mut self, from:usize, to: usize) -> Result<usize, std::io::Error> {
        let offset = from*self.typesize;
        let nitems = to - from;
        self.fpos = self.reader.seek(SeekFrom::Start(offset as u64))?;
        let nbytes = self.typesize * nitems;
        if nitems > self.buf.len() {
            //println!("trying to reallocate from {} to {}", self.buf.capacity(), nitems - self.buf.capacity());
            self.buf.reserve(nitems-self.buf.len());
        }
        let mut items_read:usize = 0;
        let bptr = self.buf.as_mut_ptr() as *mut u8;
        unsafe {
            let bufslice = std::slice::from_raw_parts_mut(bptr, nbytes) as &mut [u8];
            assert_eq!(bufslice.len(), nbytes);
            match self.reader.read(bufslice) {
                Ok(bytes_read) => {
                    items_read = bytes_read / self.typesize;
                } 
                _ => {}
            }
            self.buf.set_len(items_read);
        }
        Ok(items_read)
    }

    fn get<'a> (&'a self, index: usize) -> &C {
        &self.buf[index]
    }

    fn get_slice<'a> (&'a self, start: usize , end: usize) -> &[C] {
        &self.buf[start..end]
    }

}


pub struct XStream<R, C> 
where R: Read + Seek
{
    core: RefCell<XStreamCore<R,C>>,
}

impl<R, C> XStream<R, C> 
where R: Read + Seek
{
    pub fn new(reader: R, someval: C) -> Self {
        XStream {
            core: RefCell::new(XStreamCore::new(reader, std::mem::size_of::<C>())),
        }
    }

    pub fn get(&self, index:usize) -> &C {
        self.core.borrow_mut().read_to_buf(index, index+1);
        unsafe {
            let ptr = self.core.as_ptr();
            XStreamCore::get(&*ptr, index)
        }
    }

    pub fn get_slice(&self, start: usize, end:usize) -> &[C] {
        self.core.borrow_mut().read_to_buf(start, end);
        unsafe{
            let ptr = self.core.as_ptr();
            XStreamCore::get_slice(&*ptr, start, end)
        }
    }
}


pub trait XStreamIndex<R,C> 
where R: Read + Seek
{
    fn index<'a>(&self, xstr: &'a XStream<R,C>) -> &'a [C];
}


impl<R,C> XStreamIndex<R,C> for Range<usize> 
where R: Read + Seek
{
    fn index<'a>(&self, xstr: &'a XStream<R,C>) -> &'a [C] {
        xstr.get_slice(self.start, self.end)
    }
}

impl<R, C, I> Index<I> for XStream<R, C>
where R: Read + Seek,
      I: XStreamIndex<R, C>,
{
    type Output = [C];

    fn index(&self, index: I) -> &Self::Output {
        index.index(&self)
    }
}

#[cfg(test)] 
mod tests {
    use super::*;
    use std::io::{Write, BufWriter};



    fn test_setup() -> std::io::Result<()> {
        let mut v:Vec<u16> = vec![0];
        let mut w = BufWriter::new(File::create("test.txt")?);
        let p = v.as_ptr() as *const u8;
        for i in 0..1024 as u16 {
            v[0] = i;
            unsafe {
                let buf = std::slice::from_raw_parts(p, 2);
                w.write(buf)?;
            }
        }
        w.flush()?;
        Ok(()) 
    }

    fn test_teardown() -> std::io::Result<()> {
        std::fs::remove_file("test.txt")?;
        Ok(())
    }

    #[test]
    fn test_get() -> std::io::Result<()>{
        test_setup();
        let reader = BufReader::new(File::open("test.txt")?);
        let mut xstr = XStream::new(reader, 0 as u16);
        for i in 0..1024 as u16 {
            println!("reading element at index {}", i);
            assert_eq!(i, *xstr.get(i as usize));
        }
        test_teardown();
        Ok(()) 
    }

    #[test]
    fn test_get_slice() -> std::io::Result<()>{
        test_setup();
        let reader = BufReader::new(File::open("test.txt")?);
        let mut xstr = XStream::new(reader, 0 as u16);
        for to in 0..1024 {
            for from in 0..to {
                //println!("reading from {} to {}", from, to);
                let sl = xstr.get_slice(from, to);
                assert_eq!(to-from, sl.len());
                assert_eq!(sl[0], from as u16);
                assert_eq!(sl[sl.len()-1], (to-1) as u16);
            }
        }
        test_teardown();
        Ok(()) 
    }

}