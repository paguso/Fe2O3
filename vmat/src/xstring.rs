//use std::vec::Vec;
use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use std::iter::IntoIterator;


pub trait XString : Index<usize> + IndexMut<usize> + Deref + DerefMut + IntoIterator   {
    type CharType;

    fn len(&self) -> usize;
    fn push(&mut self, value: Self::CharType);
    fn substring(&self, begin: usize, end: usize) -> &[Self::CharType];
}

struct VecString<T> {
    v: Vec<T>,
}

impl<T> VecString<T> {
    fn new() -> VecString<T> {
        VecString {
            v: Vec::new()
        }
    }
}

impl<T> XString for VecString<T> {
    type CharType = T;

    fn len(&self) -> usize {
        self.v.len()
    }   
    
    fn push(&mut self, value: T) {
        self.v.push(value);
    }
    
    fn substring(&self, begin: usize, end: usize) -> &[Self::CharType] {
        &self.v[begin..end]        
    }
}

impl<T> Index<usize> for VecString<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    } 
}

impl<T> IndexMut<usize> for VecString<T> {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut Self::Output {
        &mut self.v[index]
    }
}

impl<T> Deref for VecString<T> {
    type Target = [T];
    fn deref (&self) -> &Self::Target {
        &self.v
    }
}

impl<T> DerefMut for VecString<T> {
    fn deref_mut (&mut self) -> &mut Self::Target {
        &mut self.v
    }
}

impl<T> IntoIterator for VecString<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
       self.v.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a VecString<T> {
    type Item = &'a T;
    type IntoIter = ::std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.v.iter()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vecstr_len() {
        let mut vstr:VecString<u8> = VecString::new();
        assert_eq!(vstr.len(), 0);
    }
    
    #[test]
    fn test_vecstr_push() {
        let mut vstr:VecString<u8> = VecString::new();
        assert_eq!(vstr.len(), 0);
        let n:usize = 10;
        for i in 0..n {
            vstr.push(i as u8);
        }
        assert_eq!(vstr.len(), n);
        for i in 0..n {
            assert_eq!(vstr[i], i as u8);
        }
        let mut i:u8 = 0;
        for &c in vstr.iter() {
            assert_eq!(c, i);
            i += 1;
        }
        vstr[0] = 2;
    }

    fn len_deref<T>(slice: &[T]) -> usize {
        println!("T len is {}", slice.len());
        slice.len()
    }

    #[test]
    fn test_vecstr_substring() {
        let mut vstr:VecString<u8> = VecString::new();
        assert_eq!(vstr.len(), 0);
        let n:usize = 10;
        for i in 0..n {
            vstr.push(i as u8);
        }
        assert_eq!(vstr.len(), n);
        let begin = 1;
        let end = 5;
        let s = vstr.substring(begin, end);
        assert_eq!(s.len(), end-begin);
        for i in 0..s.len() {
            assert_eq!(vstr[begin+i], s[i]);
        }
        assert_eq!(len_deref(&vstr), n);
        assert_eq!(len_deref(&s), end-begin);

    }

}