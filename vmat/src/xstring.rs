use std::cmp;
use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::slice::SliceIndex;

use crate::alphabet::Alphabet;

pub struct XString<C>
where C: Copy + Default
{
    v: Vec<C>,
}

impl<C> XString<C> 
where C: Copy + Default 
{ 
    pub fn new() -> Self {
        XString {
            v: Vec::new()
        }
    }
    
    pub fn repeat(len: usize, chr: C) -> Self {
        XString {
            v: vec![chr; len] 
        }
    }


    pub fn len(&self) -> usize {
        self.v.len()
    }   

    pub fn substring(&self, begin:usize, end: usize) -> &[C] {
        &self.v[begin..end]
    }
    
    pub fn push(&mut self, value: C) {
        self.v.push(value);
    }

    pub fn pop(&mut self) -> Option<C> {
        self.v.pop()
    }
    
    pub fn remove(&mut self, index: usize)  -> C {
        self.v.remove(index)
    }

    pub fn append_from_slice(&mut self, suff: &[C]) {
        self.v.extend_from_slice(suff);
    }

    pub fn truncate(&mut self, len:usize) {
        self.v.truncate(len);
    } 
}

impl<C, I> Index<I> for XString<C>
where
    C: Copy + Default,
    I: SliceIndex<[C]>,
{
    type Output = <I as SliceIndex<[C]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.v[index]
    }
}

impl<C, I> IndexMut<I> for XString<C> 
where 
    C: Copy + Default, 
    I: SliceIndex<[C]>
{
    fn index_mut<'a>(&'a mut self, index: I) -> &'a mut Self::Output {
        &mut self.v[index]
    }
}

impl<C> Deref for XString<C> 
where C: Copy + Default, 
{
    type Target = [C];
    fn deref (&self) -> &Self::Target {
        &self.v
    }
}

impl<C> DerefMut for XString<C> 
where C: Copy + Default,
{
    fn deref_mut (&mut self) -> &mut Self::Target {
        &mut self.v
    }
}



pub trait XStrRanker {
    type  CharType;
    fn rank(&self, s: &[Self::CharType]) -> usize; 
}

pub struct XStrLexRanker<C, A>
where C: Eq,  
      A: Alphabet<CharType=C>
{
    ab : Rc<A>
}

impl<C, A> XStrLexRanker<C, A> 
where C: Eq,  
      A: Alphabet<CharType=C>
{
    pub fn new(ab: Rc<A>) -> Self {
        XStrLexRanker{ab}
    }
}

impl<C, A> XStrRanker for XStrLexRanker<C, A> 
where C: Eq,  
      A: Alphabet<CharType=C>
{
    type CharType = C;
    fn rank(&self, s: &[Self::CharType]) -> usize {
        let mut r:usize = 0;
        for c in s {
            r = (r * self.ab.len()) + self.ab.ord(c).expect("Char not in alphabet");
        }
        r
    }

} 


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xstr_len() {
        let mut xstr:XString<u8> = XString::new();
        assert_eq!(xstr.len(), 0);
    }
    
    #[test]
    fn test_xstr_push() {
        let mut xstr:XString<u8> = XString::new();
        assert_eq!(xstr.len(), 0);
        let n:usize = 10;
        for i in 0..n {
            xstr.push(i as u8);
        }
        assert_eq!(xstr.len(), n);
        for i in 0..n {
            assert_eq!(xstr[i], i as u8);
        }
        let mut i:u8 = 0;
        for &c in xstr.iter() {
            assert_eq!(c, i);
            i += 1;
        }
        xstr[0] = 2;
    }

    fn len_deref<T>(slice: &[T]) -> usize {
        println!("T len is {}", slice.len());
        slice.len()
    }

    /*
    #[test]
    fn test_xstr_substring() {
        let mut xstr:XString<u8> = XString::new();
        assert_eq!(xstr.len(), 0);
        let n:usize = 10;
        for i in 0..n {
            xstr.push(i as u8);
        }
        assert_eq!(xstr.len(), n);
        let begin = 1;
        let end = 5;
        let s = xstr.substring(begin, end);
        assert_eq!(s.len(), end-begin);
        for i in 0..s.len() {
            assert_eq!(xstr[begin+i], s[i]);
        }
        assert_eq!(len_deref(&xstr), n);
        assert_eq!(len_deref(&s), end-begin);
    }
    */
    
    #[test]
    fn test_xstr_slice() {
        let mut xstr:XString<u8> = XString::new();
        assert_eq!(xstr.len(), 0);
        let n:usize = 10;
        for i in 0..n {
            xstr.push(i as u8);
        }
        let k = 4;
        for i in 0..n-k {
            let slice = &xstr[i..i+k];
        }
    }


}