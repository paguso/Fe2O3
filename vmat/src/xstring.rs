use crate::alphabet::Character;

use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::slice::SliceIndex;

use crate::alphabet::Alphabet;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct XString<C>
where
    C: Character,
{
    v: Vec<C>,
}

impl<C> XString<C>
where
    C: Character,
{
    pub fn new() -> Self {
        XString { v: Vec::new() }
    }

    pub fn repeat(len: usize, chr: C) -> Self {
        XString { v: vec![chr; len] }
    }

    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn substring(&self, begin: usize, end: usize) -> &[C] {
        &self.v[begin..end]
    }

    pub fn push(&mut self, value: C) {
        self.v.push(value);
    }

    pub fn pop(&mut self) -> Option<C> {
        self.v.pop()
    }

    pub fn remove(&mut self, index: usize) -> C {
        self.v.remove(index)
    }

    pub fn append_from_slice(&mut self, suff: &[C]) {
        self.v.extend_from_slice(suff);
    }

    pub fn truncate(&mut self, len: usize) {
        self.v.truncate(len);
    }
}

impl<C> From<&[C]> for XString<C>
where
    C: Character,
{
    fn from(src: &[C]) -> Self {
        XString { v: Vec::from(src) }
    }
}

impl<C> From<Vec<C>> for XString<C>
where
    C: Character,
{
    fn from(src: Vec<C>) -> Self {
        XString { v: src }
    }
}

impl<C, I> Index<I> for XString<C>
where
    C: Character,
    I: SliceIndex<[C]>,
{
    type Output = <I as SliceIndex<[C]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.v[index]
    }
}

impl<C, I> IndexMut<I> for XString<C>
where
    C: Character,
    I: SliceIndex<[C]>,
{
    fn index_mut<'a>(&'a mut self, index: I) -> &'a mut Self::Output {
        &mut self.v[index]
    }
}

impl<C> Deref for XString<C>
where
    C: Character,
{
    type Target = [C];
    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

impl<C> DerefMut for XString<C>
where
    C: Character,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}

pub trait XStrHasher {
    type CharType;
    fn hash(&self, s: &[Self::CharType]) -> u64;
}

pub trait XStrRollHasher: XStrHasher {
    /**
     * Compute the hash h'=h(s[1..n]) from the given h=h(s[0..n-1]) for n>=2.
     * Panics if n<2.
     */
    fn roll_hash(&self, s: &[Self::CharType], h: u64) -> u64;
}

#[derive(Debug)]
pub struct XStrLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    ab: A,
}

impl<C, A> XStrLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    pub fn new(ab: A) -> Self {
        XStrLexHasher { ab }
    }
}

impl<C, A> XStrHasher for XStrLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    type CharType = C;

    fn hash(&self, s: &[Self::CharType]) -> u64 {
        let mut r: u64 = 0;
        for c in s {
            r = (r * self.ab.len() as u64) + self.ab.ord(c).expect("Char not in alphabet") as u64;
        }
        r
    }
}

impl<C, A> XStrRollHasher for XStrLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    fn roll_hash(&self, s: &[Self::CharType], h: u64) -> u64 {
        (h * self.ab.len() as u64)
            + (self.ab.ord(&s[s.len() - 1]).expect("Char not in alphabet") as u64)
    }
}

#[derive(Debug)]
pub struct KmerLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    ab: A,
    k: usize,
    msd_pow: u64,
}

impl<C, A> KmerLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    pub fn new(ab: A, k: usize) -> Self {
        let msd_pow: u64 = (ab.len() as u64).pow(k as u32 - 1);
        KmerLexHasher { ab, k, msd_pow }
    }
}

impl<C, A> XStrHasher for KmerLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    type CharType = C;

    fn hash(&self, s: &[Self::CharType]) -> u64 {
        let mut r: u64 = 0;
        for c in s {
            r = (r * self.ab.len() as u64) + self.ab.ord(c).expect("Char not in alphabet") as u64;
        }
        r
    }
}

impl<C, A> XStrRollHasher for KmerLexHasher<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    fn roll_hash(&self, s: &[Self::CharType], h: u64) -> u64 {
        ((h - (self.ab.ord(&s[0]).expect("Char not in alphabet") as u64 * self.msd_pow))
            * (self.ab.len() as u64))
            + (self.ab.ord(&s[s.len() - 1]).expect("Char not in alphabet") as u64)
    }
}
/*
use std::fmt;
impl<C,A> fmt::Debug for KmerLexHasher<C,A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KmerLexHasher alphabet={:?}", self.ab );
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xstr_len() {
        let mut xstr: XString<u8> = XString::new();
        assert_eq!(xstr.len(), 0);
    }

    #[test]
    fn test_xstr_push() {
        let mut xstr: XString<u8> = XString::new();
        assert_eq!(xstr.len(), 0);
        let n: usize = 10;
        for i in 0..n {
            xstr.push(i as u8);
        }
        assert_eq!(xstr.len(), n);
        for i in 0..n {
            assert_eq!(xstr[i], i as u8);
        }
        let mut i: u8 = 0;
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

    #[test]
    fn test_xstr_slice() {
        let mut xstr: XString<u8> = XString::new();
        assert_eq!(xstr.len(), 0);
        let n: usize = 10;
        for i in 0..n {
            xstr.push(i as u8);
        }
        let k = 4;
        for i in 0..n - k {
            let slice = &xstr[i..i + k];
        }
    }
}
