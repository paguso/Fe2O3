use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

pub trait Alphabet 
{
    type CharType: Eq + Copy;
    fn len(&self) -> usize;
    fn chr(&self, ord: usize) -> Option<&Self::CharType>;
    fn ord(&self, chr: &Self::CharType) -> Option<usize>;
}

pub struct HashAlphabet<T> 
    where T: Hash + Eq + Copy
{
    chr_vec: Vec<T>,
    ord_map: HashMap<T, usize>
}

impl<T> HashAlphabet<T> 
    where T: Copy + Hash + Eq 
{
    pub fn new(chr_vec: Vec<T>) -> HashAlphabet<T> {
        let mut ord_map = HashMap::new();
        let mut ord = 0;
        for c in &chr_vec {
            ord_map.insert(*c, ord);
            ord += 1;
        }
        HashAlphabet{chr_vec, ord_map}
    }
}

impl<T> Alphabet for HashAlphabet<T> 
    where T: Copy + Hash + Eq 
{
    type CharType = T;

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

impl<T> Index<usize> for HashAlphabet<T> 
    where T: Copy + Hash + Eq
{
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.chr_vec[idx]
    }
}

pub struct DNAAlphabet {
    letters : [char; 4]
}

impl DNAAlphabet {
    pub fn new() -> DNAAlphabet {
        DNAAlphabet{letters: ['a','c','g','t']}
    }
}

impl Alphabet for DNAAlphabet {

    type CharType = char;

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
            'a' => Some(0),
            'c' => Some(1),
            'g' => Some(2),
            't' => Some(3),
            _ => None
        }
    }
}

impl Index<usize> for DNAAlphabet 
{
    type Output = char;
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
        assert_eq!(ab[0], 'a');
        assert_eq!(ab[1], 'c');
    }
    
    #[test] 
    fn test_dna_ab_chr() {
        let ab = DNAAlphabet::new();
        assert_eq!(*ab.chr(0).unwrap(), 'a');
        assert_eq!(*ab.chr(1).unwrap(), 'c');
        assert_eq!(ab.chr(6), None);
    }
    
    #[test] 
    fn test_dna_ab_ord() {
        let ab = DNAAlphabet::new();
        assert_eq!(ab.ord(&'a',).unwrap(), 0);
        assert_eq!(ab.ord(&'g',).unwrap(), 2);
        assert_eq!(ab.ord(&'_'), None );
    }

}