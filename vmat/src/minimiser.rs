use crate::alphabet::{Alphabet, Character};
use crate::mqueue::MQueue;
use crate::xstream::XStream;
use crate::xstring::XStrHasher;
use crate::xstring::XString;
use std::cmp::{max, min};
use std::collections::{HashMap, VecDeque};
use std::io;

fn find_minimisers<C>(
    s: &mut impl XStream<CharType = C>,
    w: usize,
    k: usize,
    ranker: &impl XStrHasher<CharType = C>,
) -> Option<Vec<usize>>
where
    C: Character,
{
    let mut window = XString::new();
    let mut wlen: usize = 0;
    let mut wpos: usize = 0;

    while wlen < w + k - 1 {
        match s.get() {
            Ok(Some(c)) => {
                window.push(c);
                wlen += 1;
            }
            _ => break,
        }
    }
    if wlen < k || w == 0 {
        return None;
    }

    let mut wscores: MQueue<(u64, usize)> = MQueue::new_min();
    // process first window
    let mut pos: usize;
    for pos in 0..wlen - k + 1 {
        wscores.push((ranker.hash(&window[pos..pos + k]), pos));
    }

    let mut minimisers: Vec<usize> = vec![];
    pos = wlen - k + 1; // last kmer pos
    let mut wmin_mask: VecDeque<bool> = vec![false; w].into_iter().collect();
    while wlen >= k {
        for wmin in wscores.xtr_iter() {
            if !wmin_mask[wmin.1 - wpos] {
                minimisers.push(wmin.1);
                wmin_mask[wmin.1 - wpos] = true;
            }
        }
        match s.get() {
            Ok(Some(c)) => {
                window.rotate_left(1);
                window[wlen - 1] = c;
                wmin_mask.rotate_left(1);
                wmin_mask[w - 1] = false;
                wscores.pop();
                wscores.push((ranker.hash(&window[wlen - k..]), pos));
                pos += 1;
                wpos += 1;
            }
            _ => {
                window.remove(0);
                wlen -= 1;
            }
        }
    }
    Some(minimisers)
}

type TMmRank = u64;

pub struct MmIndex {
    w: Vec<usize>,
    k: Vec<usize>,
    tables: Vec<HashMap<TMmRank, Vec<usize>>>,
}

impl MmIndex {
    fn new(w: Vec<usize>, k: Vec<usize>) -> MmIndex {
        let l = w.len();
        assert_eq!(l, k.len());
        MmIndex {
            w: w,
            k: k,
            tables: vec![HashMap::new(); l],
        }
    }

    fn insert(&mut self, index: usize, mmrk: TMmRank, pos: usize) {
        if !self.tables[index].contains_key(&mmrk) {
            self.tables[index].insert(mmrk, vec![pos]);
        } else {
            self.tables[index].get_mut(&mmrk).unwrap().push(pos);
        }
    }

    fn get(&self, index: usize, mmrk: TMmRank) -> Option<&[usize]> {
        let v = self.tables[index].get(&mmrk);
        if v.is_none() {
            return None;
        }
        Some(v.unwrap())
    }
}

fn index_minimisers<C>(
    s: &mut impl XStream<CharType = C>,
    w: Vec<usize>,
    k: Vec<usize>,
    ranker: &[&impl XStrHasher<CharType = C>],
) -> Result<MmIndex, io::Error>
where
    C: Character,
{
    let nidx = w.len();
    let mut window: XString<C> = XString::new();
    // maximum necessary window buffer length
    let max_wlen = w.iter().zip(k.iter()).map(|(a, b)| a + b).max().unwrap() - 1;
    let min_k = *k.iter().min().unwrap();
    println!("max_wlen={} k_min={}", max_wlen, min_k);

    let mut mmindex = MmIndex::new(w, k);
    let mut wscores: Vec<MQueue<(TMmRank, usize)>> = vec![MQueue::new_min(); nidx];

    let mut pos = 0;
    //read in first window
    while !s.eos()? {
        match s.get()? {
            Some(c) => {
                if pos >= max_wlen {
                    window.rotate_left(1);
                    window[max_wlen - 1] = c;
                } else {
                    window.push(c);
                }
                pos += 1;
            },
            None => break
        }
        for i in 0..nidx {
            if pos >= mmindex.k[i] {
                let (last_mm_rk, last_mm_pos) = match wscores[i].xtr() {
                    Some(lmm) => (lmm.0, lmm.1),
                    None => (0, 0),
                };
                let kmer_rk = ranker[i].hash(&window[window.len() - mmindex.k[i]..]);
                let kmer_pos = pos - mmindex.k[i];
                wscores[i].push((kmer_rk, kmer_pos));
                if pos > mmindex.w[i] + mmindex.k[i] - 1 {
                    wscores[i].pop();
                }
                let (cur_mm_rk, cur_mm_pos) = wscores[i].xtr().unwrap();
                if last_mm_rk != *cur_mm_rk {
                    // new minimiser
                    for &(rk, p) in wscores[i].xtr_iter() {
                        mmindex.insert(i, rk, p);
                    }
                } else if *cur_mm_rk == kmer_rk {
                    // last kmer is a new occ of same old mm
                    mmindex.insert(i, kmer_rk, kmer_pos);
                }
            }
        }
    }
    // index end minimisers
    let mut still_indexing = true;
    while still_indexing {
        still_indexing = false;
        for i in 0..nidx {
            if wscores[i].len() > 1 {
                still_indexing = true;
                let (last_mm_rk, last_mm_pos) = match wscores[i].xtr() {
                    Some(lmm) => (lmm.0, lmm.1),
                    None => (0, 0),
                };
                wscores[i].pop();
                let (cur_mm_rk, cur_mm_pos) = wscores[i].xtr().unwrap();
                if last_mm_rk != *cur_mm_rk {
                    // new minimiser
                    for &(rk, p) in wscores[i].xtr_iter() {
                        mmindex.insert(i, rk, p);
                    }
                }
            }
        }
    }
    Ok(mmindex)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::{DNAAlphabet, DNAHasher};
    use crate::xstream::XStrStream;
    use crate::xstring::{XStrLexHasher, XString};
    use std::rc::Rc;

    #[test]
    fn test_find_minimisers() {
        let dna_ab = DNAAlphabet::new();
        let w = 4;
        let k = 5;
        let ranker = XStrLexHasher::new(Rc::new(dna_ab));
        let mut src = XString::from("acgtacgtacgtacgtacgtacgtacgtacgtacgtacgt".as_bytes());
        let mut stream = XStrStream::open(src);
        let minimisers = find_minimisers(&mut stream, w, k, &ranker).unwrap();
        src = stream.close();
        for j in &minimisers {
            println!("minimiser found at position {}", j);
            assert_eq!(&src[..k], &src[*j..*j + k]);
        }
    }
    
    #[test]
    fn test_index_minimisers() {
        let dna_ab = DNAAlphabet::new();
        let w = vec![6,2,8];
        let k = vec![3,6,16];
        let lexrk = DNAHasher::new(Rc::new(dna_ab));
        let ranker = vec![&lexrk; 3];
        let mut src = XString::from("acgtacgtacgtacgtacgtacgtacgtacgtacgtacgt".as_bytes());
        let mut stream = XStrStream::open(src);
        let mmindex = index_minimisers(&mut stream, w, k, &ranker[..]).unwrap();
        src = stream.close();
        let occ = mmindex.get(0, lexrk.hash("acg".as_bytes()));
        println!("acg = {0:?}",occ);
        let occ = mmindex.get(0, lexrk.hash("cgt".as_bytes()));
        println!("cgt = {0:?}",occ);
        let occ = mmindex.get(1, lexrk.hash("cgtacg".as_bytes()));
        println!("cgtacg = {0:?}",occ);
    }
}
