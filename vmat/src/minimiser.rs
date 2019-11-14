use crate::alphabet::Character;
use crate::mqueue::MQueue;
use crate::xstream::XStream;
use crate::xstring::{XStrHasher, XStrLexHasher, XStrRollHasher};
use crate::xstring::XString;
use std::cmp::{max, min};
use std::collections::{HashMap, VecDeque};
use std::io;
use std::marker::PhantomData;
use std::mem::size_of;

type TMmRank = u64;

#[inline]
fn pred(v: &[usize], x: usize) -> Option<usize> {
    if v.len()==0 || v[0] > x {
        return None;
    }
    let mut r = v.len();
    if v[r] <= x {
        return Some(r);
    }
    let mut l = 0;
    let mut h:usize;
    // invariant: pred is in [l,r)
    while r - l > 1 {
        h = (l + r) / 2;
        if v[h] <= x {
            l = h;
        } else {
            r = h;
        }
    }
    Some(l)
}

pub struct MmIndex<C, H> 
where C: Character,
      H: XStrRollHasher<CharType=C>,
{
    ac: PhantomData<C>,
    w: Vec<usize>,
    k: Vec<usize>,
    hasher: Vec<H>,
    tables: Vec<HashMap<TMmRank, Vec<usize>>>,
    nseq: usize,
    offs: Vec<usize>
}

impl<C, H> MmIndex<C, H> 
where C: Character,
      H: XStrRollHasher<CharType=C>,
{
    pub fn new(w: &[usize], k: &[usize], hashers: Vec<H>) -> Self {
        let l = w.len();
        assert_eq!(l, k.len());
        MmIndex {
            ac: Default::default(),
            w: Vec::from(w),
            k: Vec::from(k),
            hasher: hashers,
            tables: vec![HashMap::new(); l],
            nseq: 0,
            offs: vec![0]
        }
    }

    #[inline]
    fn insert(&mut self, index: usize, mmrk: TMmRank, pos: usize) {
        if !self.tables[index].contains_key(&mmrk) {
            self.tables[index].insert(mmrk, vec![pos]);
        } else {
            self.tables[index].get_mut(&mmrk).unwrap().push(pos);
        }
    }

    fn get_abs(&self, index: usize, mmrk: TMmRank) -> Option<&[usize]> {
        match self.tables[index].get(&mmrk) {
            None => None,
            Some(v) => Some(&v)
        }
    }

    fn get_rel(&self, index: usize, mmrk: TMmRank) -> Option<(usize, &[usize])> {
        match self.tables[index].get(&mmrk) {
            None => None,
            Some(pos_list) => {
                let mut iseq = 0;
                for pos in pos_list.iter() {
                    if iseq >= self.nseq     
                }


                v.iter().map(|pos|(pred(self.offs,pos))
                Some(&v)
            }
        }
    }

    pub fn index(
        &mut self,
        s: &mut impl XStream<CharType = C>
    ) -> Result<(), io::Error>
    where
        C: Character,
    {
        let nidx = self.w.len();
        let mut window: XString<C> = XString::new();
        // maximum necessary window buffer length
        let max_wlen = self.w.iter().zip(self.k.iter()).map(|(a, b)| a + b).max().unwrap() - 1;
        let min_k = *self.k.iter().min().unwrap();
        //println!("max_wlen={} k_min={}", max_wlen, min_k);

        let mut wscores: Vec<MQueue<(TMmRank, usize)>> = vec![MQueue::new_min(); nidx];
        let offset = *self.offs.last().unwrap();

        let mut pos = 0;
        let mut slen = 0;
        let mut c = s.get()?;
        //read in first window
        while c.is_some() {
            if pos >= max_wlen {
                window.rotate_left(1);
                window[max_wlen - 1] = c.unwrap();
            } else {
                window.push(c.unwrap());
            }
            pos += 1;
            for i in 0..nidx {
                if pos >= self.k[i] {
                    let (last_mm_rk, _last_mm_pos) = match wscores[i].xtr() {
                        Some(lmm) => (lmm.0, lmm.1),
                        None => (0, 0),
                    };
                    let kmer_rk = self.hasher[i].hash(&window[window.len() - self.k[i]..]);
                    let kmer_pos = pos - self.k[i];
                    wscores[i].push((kmer_rk, kmer_pos));
                    if pos > self.w[i] + self.k[i] - 1 {
                        wscores[i].pop();
                    }
                    let (cur_mm_rk, _cur_mm_pos) = wscores[i].xtr().unwrap();
                    if last_mm_rk != *cur_mm_rk {
                        // new minimiser
                        for &(rk, p) in wscores[i].xtr_iter() {
                            self.insert(i, rk, offset+p);
                        }
                    } else if *cur_mm_rk == kmer_rk {
                        // last kmer is a new occ of same old mm
                        self.insert(i, kmer_rk, offset+kmer_pos);
                    }
                }
            }
            slen += 1;
            c = s.get()?;
        }
        // index end minimisers
        let mut still_indexing = true;
        while still_indexing {
            still_indexing = false;
            for i in 0..nidx {
                if wscores[i].len() > 1 {
                    still_indexing = true;
                    let (last_mm_rk, _last_mm_pos) = match wscores[i].xtr() {
                        Some(lmm) => (lmm.0, lmm.1),
                        None => (0, 0),
                    };
                    wscores[i].pop();
                    let (cur_mm_rk, _cur_mm_pos) = wscores[i].xtr().unwrap();
                    if last_mm_rk != *cur_mm_rk {
                        // new minimiser
                        for &(rk, p) in wscores[i].xtr_iter() {
                            self.insert(i, rk, offset+p);
                        }
                    }
                }
            }
        }
        self.offs.push(slen);
        self.nseq += 1;
        Ok(())
    }
}

/*
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

pub fn index_minimisers<C>(
    s: &mut impl XStream<CharType = C>,
    w: &[usize], // Vec<usize>,
    k: &[usize], //Vec<usize>,
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
    let mut c = s.get()?;
    //read in first window
    while c.is_some() {
        if pos >= max_wlen {
            window.rotate_left(1);
            window[max_wlen - 1] = c.unwrap();
        } else {
            window.push(c.unwrap());
        }
        pos += 1;
        for i in 0..nidx {
            if pos >= mmindex.k[i] {
                let (last_mm_rk, _last_mm_pos) = match wscores[i].xtr() {
                    Some(lmm) => (lmm.0, lmm.1),
                    None => (0, 0),
                };
                let kmer_rk = ranker[i].hash(&window[window.len() - mmindex.k[i]..]);
                let kmer_pos = pos - mmindex.k[i];
                wscores[i].push((kmer_rk, kmer_pos));
                if pos > mmindex.w[i] + mmindex.k[i] - 1 {
                    wscores[i].pop();
                }
                let (cur_mm_rk, _cur_mm_pos) = wscores[i].xtr().unwrap();
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
        c = s.get()?;
    }
    // index end minimisers
    let mut still_indexing = true;
    while still_indexing {
        still_indexing = false;
        for i in 0..nidx {
            if wscores[i].len() > 1 {
                still_indexing = true;
                let (last_mm_rk, _last_mm_pos) = match wscores[i].xtr() {
                    Some(lmm) => (lmm.0, lmm.1),
                    None => (0, 0),
                };
                wscores[i].pop();
                let (cur_mm_rk, _cur_mm_pos) = wscores[i].xtr().unwrap();
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

*/

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
        let mut src = XString::from("ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT".as_bytes());
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
        let w = vec![6, 4, 8];
        let k = vec![3, 6, 16];
        let mut letters = [
            DNAAlphabet::A,
            DNAAlphabet::C,
            DNAAlphabet::G,
            DNAAlphabet::T,
        ];
        let mut ranker = vec![];
        ranker.push(DNAHasher::new(Rc::new(DNAAlphabet::new_with_permutation(
            &letters,
        ))));
        println!("letters[0] = {0:?}", letters);
        letters.rotate_left(1);
        println!("letters[1] = {0:?}", letters);
        ranker.push(DNAHasher::new(Rc::new(DNAAlphabet::new_with_permutation(
            &letters,
        ))));
        letters.rotate_left(1);
        println!("letters[1] = {0:?}", letters);
        ranker.push(DNAHasher::new(Rc::new(DNAAlphabet::new_with_permutation(
            &letters,
        ))));
        let ranker_refs = [&ranker[0], &ranker[1], &ranker[2]];
        //                           0         1         2         3
        let mut src = XString::from("ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT".as_bytes());
        let mut stream = XStrStream::open(src);
        let mmindex = index_minimisers(&mut stream, &w, &k, &ranker_refs).unwrap();
        src = stream.close();
        let occ = mmindex.get(0, ranker[0].hash("ACG".as_bytes()));
        println!("ACG = {0:?}", occ);
        let occ = mmindex.get(0, ranker[0].hash("CGT".as_bytes()));
        println!("CGT = {0:?}", occ);
        let occ = mmindex.get(1, ranker[1].hash("ACGTAC".as_bytes()));
        println!("ACGTAC = {0:?}", occ);
        let occ = mmindex.get(1, ranker[1].hash("CGTACG".as_bytes()));
        println!("CGTACG = {0:?}", occ);
        let occ = mmindex.get(1, ranker[1].hash("GTACGT".as_bytes()));
        println!("GTACGT = {0:?}", occ);
    }
}
