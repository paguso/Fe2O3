use crate::alphabet::Alphabet;
use crate::xstring::{XStrHasher, XStrRollHasher};
use std::ops::Index;
use std::rc::Rc;

pub struct DNAAlphabet {
    letters: [u8; 4],
    ranks: [usize; 256],
}

impl DNAAlphabet {
    const NULL_RK: usize = 255usize;
    const USIZE_MASK: usize = 255usize;

    pub const A: u8 = b'A';
    pub const C: u8 = b'C';
    pub const G: u8 = b'G';
    pub const T: u8 = b'T';

    fn init_ranks(&mut self) {
        for r in self.letters.iter().enumerate() {
            self.ranks[Self::USIZE_MASK & *r.1 as usize] = r.0;
        }
    }

    pub fn new() -> DNAAlphabet {
        let mut ret = DNAAlphabet {
            letters: [Self::A, Self::C, Self::G, Self::T],
            ranks: [Self::NULL_RK; 256],
        };
        ret.init_ranks();
        ret
    }

    pub fn new_with_permutation(letters: &[u8]) -> DNAAlphabet {
        assert_eq!(letters.len(), 4);
        assert!(letters.contains(&Self::A));
        assert!(letters.contains(&Self::C));
        assert!(letters.contains(&Self::G));
        assert!(letters.contains(&Self::T));
        let mut my_letters = [0u8; 4];
        my_letters.copy_from_slice(letters);
        let mut ret = DNAAlphabet {
            letters: my_letters,
            ranks: [Self::NULL_RK; 256],
        };
        ret.init_ranks();
        ret
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
        match self.ranks[Self::USIZE_MASK & *chr as usize] {
            Self::NULL_RK => None,
            r => Some(r)
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
        DNAHasher { ab }
    }
}

impl XStrHasher for DNAHasher {
    type CharType = u8;
    fn hash(&self, s: &[Self::CharType]) -> u64 {
        let mut h: u64 = 0;
        for c in s {
            h = (h << 2)
                | self.ab.ord(c).expect("Char not in DNA alphabet") as u64;
        }
        h
    }
}

impl XStrRollHasher for DNAHasher {
    fn roll_hash(&self, s: &[Self::CharType], h: u64, c: u8) -> u64 {
        (h << 2) | self.ab.ord(&c).expect("Char not in DNA alphabet") as u64
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_dna_ab_index() {
        let ab = DNAAlphabet::new();
        assert_eq!(ab[0], DNAAlphabet::A);
        assert_eq!(ab[1], DNAAlphabet::C);
    }

    #[test]
    fn test_dna_ab_chr() {
        let ab = DNAAlphabet::new();
        assert_eq!(*ab.chr(0).unwrap(), 'A' as u8);
        assert_eq!(*ab.chr(1).unwrap(), 'C' as u8);
        assert_eq!(ab.chr(6), None);
    }

    #[test]
    fn test_dna_ab_ord() {
        let ab = DNAAlphabet::new();
        assert_eq!(ab.ord(&('A' as u8),).unwrap(), 0);
        assert_eq!(ab.ord(&('G' as u8),).unwrap(), 2);
        assert_eq!(ab.ord(&('_' as u8)), None);
    }
}
