use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

pub trait Character  : Copy + Eq {}

impl Character for u8 {}
impl Character for u16 {}
impl Character for u32 {}
impl Character for char {}

pub trait Alphabet 
{
    type CharType: Character;
    fn len(&self) -> usize;
    fn chr(&self, ord: usize) -> Option<&Self::CharType>;
    fn ord(&self, chr: &Self::CharType) -> Option<usize>;
}

pub struct HashAlphabet<C> 
    where C: Character + Hash 
{
    chr_vec: Vec<C>,
    ord_map: HashMap<C, usize>
}

impl<C> HashAlphabet<C> 
    where C:  Character + Hash 
{
    pub fn new(chr_vec: Vec<C>) -> HashAlphabet<C> {
        let mut ord_map = HashMap::new();
        let mut ord = 0;
        for c in &chr_vec {
            ord_map.insert(*c, ord);
            ord += 1;
        }
        HashAlphabet{chr_vec, ord_map}
    }
}

impl<C> Alphabet for HashAlphabet<C> 
    where C: Character + Hash  
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
            None => None
        }
    }

}

impl<C> Index<usize> for HashAlphabet<C> 
    where C: Character + Hash 
{
    type Output = C;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.chr_vec[idx]
    }
}

pub struct DNAAlphabet {
    a: u8,
    c: u8,
    g: u8,
    t: u8,
    letters : [u8; 4]
}

impl DNAAlphabet {
    pub fn new() -> DNAAlphabet {
        DNAAlphabet{
            a: 'a' as u8, 
            c: 'c' as u8, 
            g: 'g' as u8, 
            t: 't' as u8, 
            letters: ['a' as u8, 'c' as u8, 'g' as u8, 't' as u8]
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
        }
        else {
            None
        }
    }

    fn ord(&self, chr: &Self::CharType) -> Option<usize> {
        match chr {
           97u8 => Some(0),
           99u8 => Some(1),
           103u8 => Some(2),
           116u8 => Some(3),
            _ => None
        }
    }
}

impl Index<usize> for DNAAlphabet 
{
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.letters[idx]
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_hashab_index() {
        let chars = vec!['a','c','g','t'];
        let ab = HashAlphabet::new(chars);
        assert_eq!(ab[0], 'a');
        assert_eq!(ab[1], 'c');
    }
    
    #[test] 
    fn test_hashab_chr() {
        let chars = vec!['a','c','g','t'];
        let ab = HashAlphabet::new(chars);
        assert_eq!(*ab.chr(0).unwrap(), 'a');
        assert_eq!(*ab.chr(1).unwrap(), 'c');
        assert_eq!(ab.chr(6), None);
    }
    
    #[test] 
    fn test_hashab_ord() {
        let chars = vec!['a','c','g','t'];
        let ab = HashAlphabet::new(chars);
        assert_eq!(ab.ord(&'a',).unwrap(), 0);
        assert_eq!(ab.ord(&'g',).unwrap(), 2);
        assert_eq!(ab.ord(&'_'), None );
    }

    #[test] 
    fn test_dna_ab_index() {
        let ab = DNAAlphabet::new();
        assert_eq!(ab[0], 'a' as u8);
        assert_eq!(ab[1], 'c' as u8);
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
        assert_eq!(ab.ord(&('_' as u8)), None );
    }

}