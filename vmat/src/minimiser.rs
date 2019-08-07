use std::cmp;
use crate::xstring::XString as XString;
use crate::alphabet::Alphabet;


trait XStringRank {
    fn rank(&self, s: &impl XString) -> usize;
}


fn find_minimisers(s: &impl XString, w: usize, k:usize, rank: &impl XStringRank) -> Option<Vec<usize>> {
    let n = s.len();
    if n < k || w == 0 {
        // no kmer
        return None; 
    }
    
    // circular buffer to store current window kmer ranks
    let mut buf:Vec<usize> = vec![];
    let mut buf_start = 0;
    // minimiser occurrences
    let mut occ:Vec<usize> = vec![];
    // last minimiser rank
    let mut last_min_rank = std::usize::MAX;
    // last minimiser rightmost position within window
    let mut last_min_pos = 0;
    // current kmer rank
    let mut pos_rank;
    // process initial window
    println!("Processing initial window");
    for j in 0..=cmp::min(w-1, n-k) {
        pos_rank = rank.rank(s.substring(j,j+k));
        buf.push(pos_rank);
        if pos_rank <= last_min_rank { // (!) <=
            last_min_rank = pos_rank;
            last_min_pos = j;
        }      
    }
    for j in 0..=cmp::min(w-1, n-k) {
        if buf[j] == last_min_rank {
            occ.push(j);
        }
    }
    // process subsequent windows, if any
    if n > (w+k-1) {
        assert_eq!(buf.len(), w);
        // current window starts at position i
        for i in 1..=n-(w+k-1) {
            println!("Processing window at position {}",i);
            // current window differs from previous by last kmer only
            // process last kmer of the window
            pos_rank = rank.rank(&s[i+w-1..i+w-1+k]);
            buf[buf_start] = pos_rank;
            buf_start = (buf_start + 1) % w;
            if pos_rank == last_min_rank {
                // last kmer is a new occurrence of last minimiser
                // previous occurrences already accounted for
                last_min_pos = w-1;
                occ.push(i+w-1);
            }
            if pos_rank < last_min_rank {
                // last kmer of current window is a new minimiser
                last_min_rank = pos_rank;
                last_min_pos = w-1;
                for j in 0..w {
                    if buf[(buf_start+j)%w] == last_min_rank {
                        occ.push(i+j);
                    }
                }
            }
            else if last_min_pos == 0 {
                // last kmer is not the minimiser 
                // but last minimiser in no longer in the window
                // must search for new window minimiser
                last_min_rank = buf[buf_start];
                last_min_pos = 0;
                for j in 1..w {
                    if buf[(buf_start+j)%w] <= last_min_rank  { // (!) <=
                        last_min_rank = buf[(buf_start+j)%w];
                        last_min_pos = j;
                    }  
                }
                for j in 0..w {
                    if buf[(buf_start+j)%w] == last_min_rank  {
                        occ.push(i+j);
                    }  
                }
            }
            else {
                // minimiser unchanged but window moved right
                last_min_pos -= 1 ;
            }
        }
    }
    Some(occ)
}



/*
use std::cmp;

use crate::alphabet::Alphabet;

trait StrRank {
    fn rank(&self, s: &str) -> usize;
}

struct KmerLexRank<'a> {
    ab: &'a Alphabet<char>,
    k: usize,
}

impl<'a> KmerLexRank<'a> {
    fn new(k:usize, ab: &'a Alphabet<char>) -> KmerLexRank {
        KmerLexRank{k, ab}
    }
}

impl<'a> StrRank for KmerLexRank<'a> {
    fn rank(&self, s: &str) -> usize {
       let mut rnk:usize = 0;
       let mut schars = s.chars();
       for _ in 0..self.k {
            let c = schars.next().expect(
                        &format!("String length >= {} required.", self.k)
                    );
            rnk = self.ab.len() * rnk + self.ab.ord(&c).expect(
                &format!("Alphabet does not contain char '{}'.", c)
            );
       }
       rnk 
    }
}

///
/// Return the (`w`, `k`)-minimisers' positions in `s`
/// 
fn find_minimisers(s: &str, w: usize, k:usize, rank: &StrRank) -> Option<Vec<usize>> {
    let n = s.len();
    if n < k || w == 0 {
        // no kmer
        return None; 
    }
    
    // circular buffer to store current window kmer ranks
    let mut buf:Vec<usize> = vec![];
    let mut buf_start = 0;
    // minimiser occurrences
    let mut occ:Vec<usize> = vec![];
    // last minimiser rank
    let mut last_min_rank = std::usize::MAX;
    // last minimiser rightmost position within window
    let mut last_min_pos = 0;
    // current kmer rank
    let mut pos_rank;
    // process initial window
    println!("Processing initial window");
    for j in 0..=cmp::min(w-1, n-k) {
        pos_rank = rank.rank(&s[j..j+k]);
        buf.push(pos_rank);
        if pos_rank <= last_min_rank { // (!) <=
            last_min_rank = pos_rank;
            last_min_pos = j;
        }      
    }
    for j in 0..=cmp::min(w-1, n-k) {
        if buf[j] == last_min_rank {
            occ.push(j);
        }
    }
    // process subsequent windows, if any
    if n > (w+k-1) {
        assert_eq!(buf.len(), w);
        // current window starts at position i
        for i in 1..=n-(w+k-1) {
            println!("Processing window at position {}",i);
            // current window differs from previous by last kmer only
            // process last kmer of the window
            pos_rank = rank.rank(&s[i+w-1..i+w-1+k]);
            buf[buf_start] = pos_rank;
            buf_start = (buf_start + 1) % w;
            if pos_rank == last_min_rank {
                // last kmer is a new occurrence of last minimiser
                // previous occurrences already accounted for
                last_min_pos = w-1;
                occ.push(i+w-1);
            }
            if pos_rank < last_min_rank {
                // last kmer of current window is a new minimiser
                last_min_rank = pos_rank;
                last_min_pos = w-1;
                for j in 0..w {
                    if buf[(buf_start+j)%w] == last_min_rank {
                        occ.push(i+j);
                    }
                }
            }
            else if last_min_pos == 0 {
                // last kmer is not the minimiser 
                // but last minimiser in no longer in the window
                // must search for new window minimiser
                last_min_rank = buf[buf_start];
                last_min_pos = 0;
                for j in 1..w {
                    if buf[(buf_start+j)%w] <= last_min_rank  { // (!) <=
                        last_min_rank = buf[(buf_start+j)%w];
                        last_min_pos = j;
                    }  
                }
                for j in 0..w {
                    if buf[(buf_start+j)%w] == last_min_rank  {
                        occ.push(i+j);
                    }  
                }
            }
            else {
                // minimiser unchanged but window moved right
                last_min_pos -= 1 ;
            }
        }
    }
    Some(occ)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::DNAAlphabet;

    #[test]
    fn test_find_minimisers() {
        let dna_ab = DNAAlphabet::new();
        let w = 4;
        let k = 5;
        let ranker = KmerLexRank::new(k, &dna_ab);
        let s = String::from("acgtacgtacgtacgtacgtacgtacgtacgtacgtacgt");
        match find_minimisers(&s, w, k, &ranker) {
            Some(occ) => {
                for j in &occ {
                    assert_eq!(&s[..k], &s[*j..*j+k]);
                    println!("Found minimiser {} at position {}", &s[*j..*j+k], *j);
                }
            },
            None => assert!(false, "No minimisers found. Should have found some."),
        }
        let w = 1;
        let k = 500;
        let ranker = KmerLexRank::new(k, &dna_ab);
        assert_eq!(find_minimisers(&s, w, k, &ranker), None); 
        let w = 0;
        let k = 5;
        let ranker = KmerLexRank::new(k, &dna_ab);
        assert_eq!(find_minimisers(&s, w, k, &ranker), None); 
        let w = 5;
        let k = 0;
        let ranker = KmerLexRank::new(k, &dna_ab);
        match find_minimisers(&s, w, k, &ranker) {
            Some(occ) => {
                for j in &occ {
                    assert_eq!(&s[..k], &s[*j..*j+k]);
                    println!("Found minimiser {} at position {}", &s[*j..*j+k], *j);
                }
            },
            None => println!("No minimisers found"),
        }
    }
}
*/