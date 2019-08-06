//use std::vec::Vec;
use std::ops::Index;
use std::ops::IndexMut;
use std::iter::IntoIterator;

pub trait XString<T> : Index<usize> + IndexMut<usize> + IntoIterator {
    fn len(&self) -> usize;
    fn push(&mut self, value: T);
}


struct VString<T> {
    v: Vec<T>,
}

enum VEString {
    V8String(Vec<u8>),
    V16String(Vec<u16>),
    V32String(Vec<u32>),
    V64String(Vec<u64>),
    V128String(Vec<u128>)
}

impl<T> VString<T> {
    fn new() -> VString<T> {
        VString {
            v: Vec::new()
        }
    }
}

impl<T> XString<T> for VString<T> {
    
    fn len(&self) -> usize {
        self.v.len()
    }   
    
    fn push(&mut self, value: T) {
        self.v.push(value);
    }
}

impl<T> Index<usize> for VString<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    } 
}

impl<T> IndexMut<usize> for VString<T> {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut Self::Output {
        &mut self.v[index]
    }
}

impl<T> IntoIterator for VString<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.v.into_iter()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vstr_len() {
        let mut vstr:VString<u8> = VString::new();
        assert_eq!(vstr.len(), 0);
    }
    
    #[test]
    fn test_vstr_push() {
        let mut vstr:VString<u8> = VString::new();
        assert_eq!(vstr.len(), 0);
        let n:usize = 10;
        for i in 0..n {
            vstr.push(i as u8);
        }
        assert_eq!(vstr.len(), n);
        for i in 0..n {
            assert_eq!(vstr[i], i as u8);
        }
        vstr[0] = 2;
    }
}