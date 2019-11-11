use crate::alphabet::Alphabet;
use crate::xstring::{XStrHasher, XStrRollHasher};
use std::ops::Index;
use std::rc::Rc;

pub struct DNAAlphabet {
    letters: [u8; 4],
    ranks: [usize; 64],
}

impl DNAAlphabet {
    pub const A: u8 = b'A';
    pub const C: u8 = b'C';
    pub const G: u8 = b'G';
    pub const T: u8 = b'T';

    fn init_ranks(&mut self) {
        for r in self.letters.iter().enumerate() {
            self.ranks[*r.1 as usize - 65] = r.0;
        }
    }

    pub fn new() -> DNAAlphabet {
        let mut ret = DNAAlphabet {
            letters: [Self::A, Self::C, Self::G, Self::T],
            ranks: [0usize; 64],
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
            ranks: [0usize; 64],
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
        if *chr == self.letters[0] {
            return Some(0);
        } else if *chr == self.letters[1] {
            return Some(1);
        } else if *chr == self.letters[2] {
            return Some(2);
        } else if *chr == self.letters[3] {
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
            h = (h << 2)
                | self.ab.ord(c).expect(&format!(
                    "Char '{0}' not found in alphabet {1:?}",
                    c, self.ab.letters
                )) as u64;
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
