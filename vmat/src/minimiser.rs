use crate::alphabet::Character;
use crate::mqueue::MQueue;
use crate::xstream::{XStrStream, XStream};
use crate::xstring::XString;
use crate::xstring::{KmerXStrLexHasher, XStrHasher, XStrLexHasher, XStrRollHasher};
use std::cmp::{max, min};
use std::collections::{HashMap, VecDeque};
use std::io;
use std::marker::PhantomData;
use std::mem::size_of;

type TMmRank = u64;

/**
 * The minimiser index is as associative array that keeps references to the
 * occurrences of `(w,k)`-minimisers in indexed sequences.  
 * The index has `m` `(w,k)` pairs of minimiser specification parameteres.
 * Let us call `(W,K)`-mimimisers all the minimisers with such characteristics.
 * The index holds the references to the `(W,K)`-minimisers of `n` indexed
 * sequences.
 */
pub struct MmIndex<'a, C, H>
where
    C: Character,
    H: XStrRollHasher<CharType = C>,
{
    ac: PhantomData<C>,
    w: Vec<usize>,
    k: Vec<usize>,
    hasher: Vec<&'a H>,
    tables: Vec<HashMap<TMmRank, Vec<usize>>>,
    nseq: usize,
    offs: Vec<usize>,
}

impl<'a, C, H> MmIndex<'a, C, H>
where
    C: Character,
    H: XStrRollHasher<CharType = C>,
{
    pub fn new(w: &[usize], k: &[usize], hashers: &[&'a H]) -> Self {
        let l = w.len();
        assert_eq!(l, k.len());
        assert!(*w.iter().min_by(|x, y| x.cmp(y)).unwrap() > 0);
        assert!(*k.iter().min_by(|x, y| x.cmp(y)).unwrap() > 0);
        MmIndex {
            ac: Default::default(),
            w: Vec::from(w),
            k: Vec::from(k),
            hasher: Vec::from(hashers),
            tables: vec![HashMap::new(); l],
            nseq: 0,
            offs: vec![0],
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
            Some(v) => Some(&v),
        }
    }

    fn get_seq_lengths(&self) -> &[usize] {
        &self.offs[1..]
    }

    pub fn index_xstr(&mut self, s: &XString<C>) -> Result<(), io::Error> {
        let mut stream = XStrStream::open(s);
        self.index(&mut stream)
    }

    pub fn index(&mut self, s: &mut impl XStream<CharType = C>) -> Result<(), io::Error>
    where
        C: Character,
    {
        let nidx = self.w.len();
        let mut window: XString<C> = XString::new();
        // maximum necessary window buffer length
        // (!) all w,k are > 0 but, when w=1, the window has only one kmer.
        //     because we need the previous last window kmer to
        //     roll the hash value, we make sure that the window will
        //     will always contain at least two kmers for any (w,k).
        let max_wlen = self
            .w
            .iter()
            .zip(self.k.iter())
            .map(|(a, b)| a + b)
            .max()
            .unwrap(); // (!) if w[i] == 1, w[i]+k[i] == k[i]+1, which holds two kmers

        let mut wscores: Vec<MQueue<(TMmRank, usize)>> = vec![MQueue::new_min(); nidx];
        let mut last_kmer_rk = vec![0 as TMmRank; nidx];
        let offset = *self.offs.last().unwrap();

        let mut pos = 0;
        while let Some(c) = s.get()? {
            if pos >= max_wlen {
                window.rotate_left(1);
                window[max_wlen - 1] = c;
            } else {
                //read in first window
                window.push(c);
            }
            pos += 1;
            for i in 0..nidx {
                if pos == self.k[i] {
                    let kmer_rk = self.hasher[i].hash(&window[window.len() - self.k[i]..]);
                    last_kmer_rk[i] = kmer_rk;
                    wscores[i].push((kmer_rk, pos - self.k[i]));
                    // initial end minimisers are all indexed
                    self.insert(i, kmer_rk, offset + pos - self.k[i]);
                //println!("pos={} kmer_rk={}", pos, kmer_rk);
                } else if pos > self.k[i] {
                    // get previous windows minimiser
                    let (last_mm_rk, _last_mm_pos) = wscores[i].xtr().unwrap().clone();
                    // compute new last kmer rank and add it to the new window
                    let kmer_rk = self.hasher[i].roll_hash(
                        &window[window.len() - self.k[i] - 1..window.len() - 1],
                        last_kmer_rk[i],
                        c,
                    );
                    //let kmer_rk_bf = self.hasher[i].hash(&window[window.len() - self.k[i]..]);
                    //println!(
                    //    "k={} pos={} kmer_rk={} kmer_rk_bf = {}",
                    //    self.k[i], pos, kmer_rk, kmer_rk_bf
                    //);
                    //assert_eq!(kmer_rk, kmer_rk_bf);
                    let kmer_pos = pos - self.k[i];
                    last_kmer_rk[i] = kmer_rk;
                    wscores[i].push((kmer_rk, kmer_pos));
                    // dequeue the first kmer of previous window if it is full
                    if pos > self.w[i] + self.k[i] - 1 {
                        wscores[i].pop();
                    }
                    // get new window miminiser
                    let (cur_mm_rk, _cur_mm_pos) = wscores[i].xtr().unwrap().clone();
                    if self.w[i] == 1 || last_mm_rk != cur_mm_rk {
                        // new minimiser
                        for &(rk, p) in wscores[i].xtr_iter() {
                            self.insert(i, rk, offset + p);
                        }
                    } else if cur_mm_rk == kmer_rk {
                        // last kmer is a new occ of same old mm
                        self.insert(i, kmer_rk, offset + kmer_pos);
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
                    let (last_mm_rk, _last_mm_pos) = wscores[i].xtr().unwrap().clone();
                    wscores[i].pop();
                    let (cur_mm_rk, _cur_mm_pos) = wscores[i].xtr().unwrap().clone();
                    if last_mm_rk != cur_mm_rk {
                        // new minimiser
                        for &(rk, p) in wscores[i].xtr_iter() {
                            self.insert(i, rk, offset + p);
                        }
                    }
                }
            }
        }
        self.offs.push(pos);
        self.nseq += 1;
        //println!("nseq={0} offs={1:?}", self.nseq, self.offs);
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

    /*
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
    */

    #[test]
    fn test_index_minimisers() {
        let w = vec![6, 4, 8];
        let k = vec![3, 6, 16];
        println!("w={0:?}", w);
        println!("k={0:?}", k);
        let mut letters = [
            DNAAlphabet::A,
            DNAAlphabet::C,
            DNAAlphabet::G,
            DNAAlphabet::T,
        ];
        let mut ranker = vec![];
        ranker.push(KmerXStrLexHasher::new(
            Rc::new(DNAAlphabet::new_with_permutation(&letters)),
            k[0],
        ));
        println!("letters[0] = {0:?}", letters);
        letters.rotate_left(1);
        println!("letters[1] = {0:?}", letters);
        ranker.push(KmerXStrLexHasher::new(
            Rc::new(DNAAlphabet::new_with_permutation(&letters)),
            k[1],
        ));
        letters.rotate_left(1);
        println!("letters[2] = {0:?}", letters);
        ranker.push(KmerXStrLexHasher::new(
            Rc::new(DNAAlphabet::new_with_permutation(&letters)),
            k[2],
        ));
        let ranker_refs = [&ranker[0], &ranker[1], &ranker[2]];
        //                           0         1         2         3
        let mut mmindex = MmIndex::new(&w, &k, &ranker_refs);

        let src = XString::from("ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT".as_bytes());
        let mut stream = XStrStream::open(&src);
        mmindex.index(&mut stream).expect("Error indexing stream");
        stream.close();

        let occ = mmindex.get_abs(0, ranker[0].hash("ACG".as_bytes()));
        println!("ACG = {0:?}", occ);
        let occ = mmindex.get_abs(0, ranker[0].hash("CGT".as_bytes()));
        println!("CGT = {0:?}", occ);
        let occ = mmindex.get_abs(1, ranker[1].hash("ACGTAC".as_bytes()));
        println!("ACGTAC = {0:?}", occ);
        let occ = mmindex.get_abs(1, ranker[1].hash("CGTACG".as_bytes()));
        println!("CGTACG = {0:?}", occ);
        let occ = mmindex.get_abs(1, ranker[1].hash("GTACGT".as_bytes()));
        println!("GTACGT = {0:?}", occ);

        stream = XStrStream::open(&src);
        mmindex.index(&mut stream).expect("Error indexing stream");
        stream.close();

        let occ = mmindex.get_abs(0, ranker[0].hash("ACG".as_bytes()));
        println!("ACG = {0:?}", occ);
        let occ = mmindex.get_abs(0, ranker[0].hash("CGT".as_bytes()));
        println!("CGT = {0:?}", occ);
        let occ = mmindex.get_abs(1, ranker[1].hash("ACGTAC".as_bytes()));
        println!("ACGTAC = {0:?}", occ);
        let occ = mmindex.get_abs(1, ranker[1].hash("CGTACG".as_bytes()));
        println!("CGTACG = {0:?}", occ);
        let occ = mmindex.get_abs(1, ranker[1].hash("GTACGT".as_bytes()));
        println!("GTACGT = {0:?}", occ);
    }
}
