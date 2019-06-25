//use std::vec::Vec;
use std::ops::Index;


pub trait XString<T> : Index<usize> {
    fn len(&self) -> usize;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vstr_len() {
        let mut vstr:VString<u8> = VString::new();
        assert_eq!(vstr.len(), 0);
    }
}