use crate::alphabet::Alphabet;
use crate::xstring::{XStrHasher, XStrRollHasher};
use std::ops::Index;
use std::rc::Rc;

pub struct DNAAlphabet {
    letters: [u8; 4],
}

impl DNAAlphabet {
    pub const a: u8 = b'a';
    pub const c: u8 = b'c';
    pub const g: u8 = b'g';
    pub const t: u8 = b't';

    pub fn new() -> DNAAlphabet {
        DNAAlphabet {
            letters: [Self::a, Self::c, Self::g, Self::t],
        }
    }

    pub fn new_with_permutation(letters: &[u8]) -> DNAAlphabet {
        assert_eq!(letters.len(), 4);
        assert!(letters.contains(&Self::a));
        assert!(letters.contains(&Self::c));
        assert!(letters.contains(&Self::g));
        assert!(letters.contains(&Self::t));
        let mut my_letters = [0u8; 4];
        my_letters.copy_from_slice(letters);
        DNAAlphabet {
            letters: my_letters,
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
        if (*chr == self.letters[0]) {
            return Some(0);
        } else if (*chr == self.letters[1]) {
            return Some(1);
        } else if (*chr == self.letters[2]) {
            return Some(2);
        } else if (*chr == self.letters[3]) {
            return Some(3);
        } else {
            return None;
        };
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
            h = (h << 2) | self.ab.ord(c).expect("Char not in alphabet") as u64;
        }
        h
    }
}

impl XStrRollHasher for DNAHasher {
    fn roll_hash(&self, h: &mut u64, c: u8) {
        *h = (*h << 2) | self.ab.ord(&c).expect("Char not in alphabet") as u64;
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
