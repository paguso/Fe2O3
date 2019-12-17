use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;

use crate::alphabet::Character;
use crate::mqueue::MQueue;
use crate::xstream::{XStrStream, XStream};
use crate::xstring::{XString, XStrRollHasher};


type TMmRank = u64;

/**
 * The minimiser index is as associative array that keeps references to the
 * positions of `(w,k)`-minimisers in indexed sequences.  
 * The index has `m` `(w,k)` pairs of minimiser specification parameteres.
 * Let us call `(W,K)`-mimimisers all the minimisers with such characteristics.
 * The index holds the references to the `(W,K)`-minimisers of `n` indexed
 * sequences. 
 * Querying the index for a kmer `X` returns the positions of the occurrences
 * of `X` provided it is a minimiser of any indexed sequences.
 * The returned positions are *absolute*, that is, they indicate the start
 * positions of `X` in the sequence corresponding to the concatenation of
 * all indexed sequences.
 */
pub struct MmIndex<'a, C, H>
where
    C: Character,
    H: XStrRollHasher<CharType = C>,
{
    ac: PhantomData<C>,
    w: Vec<usize>,
    k: Vec<usize>,
    max_wlen: usize,
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
        // maximum necessary window buffer length
        // (!) all w,k are > 0 but, when w=1, the window has only one kmer.
        //     because we need the previous last window kmer to
        //     roll the hash value, we make sure that the window will
        //     will always contain at least two kmers for any (w,k).
        let max_wlen = w
            .iter()
            .zip(k.iter())
            .map(|(a, b)| a + b)
            .max()
            .unwrap(); // (!) if w[i] == 1, w[i]+k[i] == k[i]+1, which holds two kmers

        MmIndex {
            ac: Default::default(),
            w: Vec::from(w),
            k: Vec::from(k),
            max_wlen,
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

    pub fn index_xstr(&mut self, s: &XString<C>) -> Result<(), io::Error> 
    where
        C: Character,
    {
        let mut stream = XStrStream::open(s);
        self.index(&mut stream)
    }

    pub fn index(&mut self, s: &mut impl XStream<CharType = C>) -> Result<(), io::Error>
    where
        C: Character,
    {
        let nidx = self.w.len();
        let offset = *self.offs.last().unwrap();
        let max_win_len = self.max_wlen;
        
        let mut window: XString<C> = XString::new(); // text window containing all kmers
        let mut win_rks: Vec<MQueue<(TMmRank, usize)>> = vec![MQueue::new_min(); nidx]; // window kmers ranks
        let mut prev_mm_rk = vec![0 as TMmRank; nidx]; // rank of previous window minimiser
        let mut prev_right_rk = vec![0 as TMmRank; nidx]; // rank of previous window rightmost kmer

        let mut pos = 0;
        while let Some(c) = s.get()? {
            // perpare window
            if pos >= max_win_len {
                window.rotate_left(1);
                window[max_win_len - 1] = c;
            } else {
                //read in first window
                window.push(c);
            }
            pos += 1;
            for i in 0..nidx {
                if pos == self.k[i] {
                    let kmer_rk = self.hasher[i].hash(&window[window.len() - self.k[i]..]);
                    prev_right_rk[i] = kmer_rk;
                    prev_mm_rk[i] = kmer_rk; 
                    win_rks[i].push((kmer_rk, pos - self.k[i]));
                    // initial end minimisers are all indexed
                    self.insert(i, kmer_rk, offset + pos - self.k[i]);
                } else if pos > self.k[i] {
                    // get previous windows minimiser
                    // let (last_mm_rk, _last_mm_pos) = win_rks[i].xtr().unwrap().clone();
                    // compute new last kmer rank and add it to the new window
                    let kmer_rk = self.hasher[i].roll_hash(
                        &window[window.len() - self.k[i] - 1..window.len() - 1],
                        prev_right_rk[i],
                        c,
                    );
                    let kmer_pos = pos - self.k[i];
                    prev_right_rk[i] = kmer_rk;
                    // dequeue the first kmer of previous window if it is full
                    if pos > self.w[i] + self.k[i] - 1 {
                        win_rks[i].pop();
                    }
                    // and add new kmer
                    win_rks[i].push((kmer_rk, kmer_pos));
                    // then get current window miminiser
                    let cur_mm_rk = win_rks[i].xtr().unwrap().0;
                    if self.w[i] == 1 || prev_mm_rk[i] != cur_mm_rk {
                        // new minimiser. add all its occurrences
                        for &(rk, p) in win_rks[i].xtr_iter() {
                            self.insert(i, rk, offset + p);
                        }
                        prev_mm_rk[i] = cur_mm_rk;
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
                if win_rks[i].len() > 1 {
                    still_indexing = true;
                    let (last_mm_rk, _last_mm_pos) = win_rks[i].xtr().unwrap().clone();
                    win_rks[i].pop();
                    let (cur_mm_rk, _cur_mm_pos) = win_rks[i].xtr().unwrap().clone();
                    if last_mm_rk != cur_mm_rk {
                        // new minimiser
                        for &(rk, p) in win_rks[i].xtr_iter() {
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::{DNAAlphabet};
    use crate::xstream::XStrStream;
    use crate::xstring::{KmerXStrLexHasher, XStrHasher, XString};
    use std::rc::Rc;
    
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
