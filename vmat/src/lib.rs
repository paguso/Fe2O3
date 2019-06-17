use std::ops::Index;
use std::collections::HashMap;
use std::hash::Hash;


trait Alphabet<T> 
    where T: Eq
{
    fn len(&self) -> usize;
    fn chr(&self, ord: usize) -> Option<&T>;
    fn ord(&self, chr: &T) -> Option<usize>;
}

struct HashAlphabet<'a, T> 
    where T: Hash + Eq  
{
    chr_vec: Vec<T>,
    ord_map: HashMap<&'a T, usize>
}

impl<'a, T> HashAlphabet<'a, T> 
    where T: Hash + Eq 
{
    fn new(chr_vec: Vec<T>) -> HashAlphabet<'a, T> {
        //let mut ord_map = HashMap::new();
        let mut ret = HashAlphabet{chr_vec, ord_map:HashMap::new()};
        let mut ord = 0;
        for c in ret.chr_vec.iter() {
            ret.ord_map.insert(&c, ord);
            ord += 1;
        }
        ret
    }
}

impl<'a, T> Alphabet<T> for HashAlphabet<'a, T> 
    where T: Hash + Eq 
{
    fn len(&self) -> usize {
        self.chr_vec.len()
    }

    fn chr(&self, ord: usize) -> Option<&T> {
        self.chr_vec.get(ord)
    }

    fn ord(&self, chr: &T) -> Option<usize> {
        match self.ord_map.get(chr) {
            Some(val) => Some(*val),
            None => None
        }
    }

}


impl<'a, T> Index<usize> for HashAlphabet<'a, T> 
    where T: Hash + Eq
{
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.chr_vec[idx]
    }
}


trait KmerHasher {
    fn hash(kmer: &str) -> usize;
}

struct LexKmerHasher {
    alphabet: String
}




fn find_minimisers(src: &str, w: usize, k:usize) -> Vec<usize> {
    let mut ret = vec![];
    ret
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_ab_index() {
        let chars = vec!['a','c','g','t'];
        let rank = HashRank::new(&chars);
        let ab = Alphabet::new(chars, rank);
        assert_eq!(ab[0], 'a');
        assert_eq!(ab[1], 'c');
    }
    
    #[test] 
    fn test_ab_rank() {
        let chars = vec!['a','c','g','t'];
        let rank = HashRank::new(&chars);
        let ab = Alphabet::new(chars, rank);
        assert_eq!(ab.rank('a',).unwrap(), 0);
        assert_eq!(ab.rank('g',).unwrap(), 2);
        assert_eq!(ab.rank('_'), None );
    }
}