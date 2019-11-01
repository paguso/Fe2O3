use std::ops::Index;
use std::rc::Rc;
use crate::alphabet::{Alphabet};
use crate::xstring::{XStrHasher, XStrRollHasher};

pub struct DNAAlphabet {
    letters: [u8; 4],
}

impl DNAAlphabet {
    const a: u8 = b'a';
    const c: u8 = b'c';
    const g: u8 = b'g';
    const t: u8 = b't';

    pub fn new() -> DNAAlphabet {
        DNAAlphabet {
            letters: [Self::a, Self::c, Self::g, Self::t],
        }
    }
}

impl Alphabet for DNAAlphabet {
    type CharType = u8;

    fn len(&self) -> usize {
        4
    }

    fn chr(&self, ord: usize) -> Option<&Self::CharType> {
        if ord < 4 {
            Some(&self.letters[ord])
        } else {
            None
        }
    }

    fn ord(&self, chr: &Self::CharType) -> Option<usize> {
        match chr {
            &Self::a => Some(0),
            &Self::c => Some(1),
            &Self::g => Some(2),
            &Self::t => Some(3),
            _ => None,
        }
    }
}


impl Index<usize> for DNAAlphabet {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.letters[idx]
    }
}


pub struct DNAHasher {
    ab: Rc<DNAAlphabet>,
}


impl DNAHasher {
    pub fn new(ab: Rc<DNAAlphabet>) -> DNAHasher {
        DNAHasher {
            ab
        }
    }
}

impl XStrHasher for DNAHasher {
    type CharType = u8;
    fn hash (&self,  s:&[Self::CharType]) -> u64
    {
        let mut h: u64 = 0;
        for c in s {
            h = (h << 2) | self.ab.ord(c).expect("Char not in alphabet") as u64;
        }
        h
    }
}

impl XStrRollHasher for DNAHasher {
    fn roll_hash (&self, h: &mut u64, c: u8) 
    {
        *h = (*h << 2 ) | self.ab.ord(&c).expect("Char not in alphabet") as u64;
    }
}


mod tests {
    use super::*;

    #[test]
    fn test_dna_ab_index() {
        let ab = DNAAlphabet::new();
        assert_eq!(ab[0], DNAAlphabet::a);
        assert_eq!(ab[1], DNAAlphabet::c);
    }

    #[test]
    fn test_dna_ab_chr() {
        let ab = DNAAlphabet::new();
        assert_eq!(*ab.chr(0).unwrap(), 'a' as u8);
        assert_eq!(*ab.chr(1).unwrap(), 'c' as u8);
        assert_eq!(ab.chr(6), None);
    }

    #[test]
    fn test_dna_ab_ord() {
        let ab = DNAAlphabet::new();
        assert_eq!(ab.ord(&('a' as u8),).unwrap(), 0);
        assert_eq!(ab.ord(&('g' as u8),).unwrap(), 2);
        assert_eq!(ab.ord(&('_' as u8)), None);
    }
}
