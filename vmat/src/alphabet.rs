use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

pub trait Character: Copy + Eq + Hash {}

impl Character for u8 {}
impl Character for u16 {}
impl Character for u32 {}
impl Character for char {}

pub trait Alphabet {
    type CharType: Character;
    fn len(&self) -> usize;
    fn chr(&self, ord: usize) -> Option<&Self::CharType>;
    fn ord(&self, chr: &Self::CharType) -> Option<usize>;
}

pub struct HashAlphabet<C>
where
    C: Character + Hash,
{
    chr_vec: Vec<C>,
    ord_map: HashMap<C, usize>,
}

impl<C> HashAlphabet<C>
where
    C: Character + Hash,
{
    pub fn new(chr_vec: Vec<C>) -> HashAlphabet<C> {
        let mut ord_map = HashMap::new();
        let mut ord = 0;
        for c in &chr_vec {
            ord_map.insert(*c, ord);
            ord += 1;
        }
        HashAlphabet { chr_vec, ord_map }
    }
}

impl<C> Alphabet for HashAlphabet<C>
where
    C: Character + Hash,
{
    type CharType = C;

    fn len(&self) -> usize {
        self.chr_vec.len()
    }

    fn chr(&self, ord: usize) -> Option<&Self::CharType> {
        self.chr_vec.get(ord)
    }

    fn ord(&self, chr: &Self::CharType) -> Option<usize> {
        match self.ord_map.get(chr) {
            Some(val) => Some(*val),
            None => None,
        }
    }
}

impl<C> Index<usize> for HashAlphabet<C>
where
    C: Character + Hash,
{
    type Output = C;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.chr_vec[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashab_index() {
        let chars = vec!['a', 'c', 'g', 't'];
        let ab = HashAlphabet::new(chars);
        assert_eq!(ab[0], 'a');
        assert_eq!(ab[1], 'c');
    }

    #[test]
    fn test_hashab_chr() {
        let chars = vec!['a', 'c', 'g', 't'];
        let ab = HashAlphabet::new(chars);
        assert_eq!(*ab.chr(0).unwrap(), 'a');
        assert_eq!(*ab.chr(1).unwrap(), 'c');
        assert_eq!(ab.chr(6), None);
    }

    #[test]
    fn test_hashab_ord() {
        let chars = vec!['a', 'c', 'g', 't'];
        let ab = HashAlphabet::new(chars);
        assert_eq!(ab.ord(&'a',).unwrap(), 0);
        assert_eq!(ab.ord(&'g',).unwrap(), 2);
        assert_eq!(ab.ord(&'_'), None);
    }
}
