use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

use bus::Bus;

use crate::alphabet::{Alphabet, Character};
use crate::mqueue::MQueue;
use crate::xstream::{XStrStream, XStream};
use crate::xstring::{KmerLexHasher, XStrHasher, XStrRollHasher, XString};

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
pub struct MmIndex<C, A>
where
    C: Character,
    A: Alphabet<CharType = C>,
{
    ab: Vec<A>,
    w: Vec<usize>,
    k: Vec<usize>,
    hashers: Vec<KmerLexHasher<C, A>>,
    tables: Vec<HashMap<TMmRank, Vec<usize>>>,
    nseq: usize,
    offs: Vec<usize>,
}

impl<C, A> MmIndex<C, A>
where
    A: Alphabet<CharType = C>,
    C: Character,
    //H: XStrRollHasher<CharType = C>,
{
    pub fn new(w: &[usize], k: &[usize], ab: A) -> Self {
        let l = w.len();
        assert_eq!(l, k.len());
        assert!(*w.iter().min_by(|x, y| x.cmp(y)).unwrap() > 0);
        assert!(*k.iter().min_by(|x, y| x.cmp(y)).unwrap() > 0);

        // generate alphabet permutations and hashers
        let mut ab = vec![ab];
        for i in 1..l {
            ab.push(ab[i - 1].clone());
        }
        let hashers: Vec<KmerLexHasher<C, A>> = ab
            .iter()
            .enumerate()
            .map(|(i, a)| KmerLexHasher::new(a.clone(), k[i]))
            .collect();

        MmIndex {
            //_c: Default::default(),
            ab: ab,
            w: Vec::from(w),
            k: Vec::from(k),
            hashers: hashers,
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

        // maximum necessary window buffer length
        // (!) all w,k are > 0 but, when w=1, the window has only one kmer.
        //     because we need the previous last window kmer to
        //     roll the hash value, we make sure that the window will
        //     will always contain at least two kmers for any (w,k).
        let max_win_len = self
            .w
            .iter()
            .zip(self.k.iter())
            .map(|(a, b)| a + b)
            .max()
            .unwrap(); // (!) if w[i] == 1, w[i]+k[i] == k[i]+1, which holds two kmers

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
                    let kmer_rk = self.hashers[i].hash(&window[window.len() - self.k[i]..]);
                    prev_right_rk[i] = kmer_rk;
                    prev_mm_rk[i] = kmer_rk;
                    win_rks[i].push((kmer_rk, pos - self.k[i]));
                    // initial end minimisers are all indexed
                    self.insert(i, kmer_rk, offset + pos - self.k[i]);
                } else if pos > self.k[i] {
                    // get previous windows minimiser
                    // let (last_mm_rk, _last_mm_pos) = win_rks[i].xtr().unwrap().clone();
                    // compute new last kmer rank and add it to the new window
                    let kmer_rk = self.hashers[i]
                        .roll_hash(&window[window.len() - self.k[i] - 1..], prev_right_rk[i]);
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

    pub fn index_par(&mut self, s: &mut impl XStream<CharType = C>) -> Result<(), io::Error> {
        let nidx = self.w.len();
        let offset = *self.offs.last().unwrap();
        let mut thread_handlers = vec![];
        let mut strlen = 0;

        // temporarily extract and pack inner tables and hashers to be sent to threads
        // but keep references to be restored at the end
        let mut tbl_refs: Vec<Arc<Mutex<HashMap<TMmRank, Vec<usize>>>>> = vec![];
        let mut hsh_refs: Vec<Arc<KmerLexHasher<C, A>>> = vec![];

        {
            // the bus tx has to fall out of this inner scope to be dropped, causing
            // the rx´s thread loops to terminate due to no more incoming data
            let mut char_tx: Bus<C> = Bus::new(1024);

            while self.tables.len() > 0 {
                tbl_refs.push(Arc::new(Mutex::new(self.tables.remove(0))));
            }
            while self.hashers.len() > 0 {
                hsh_refs.push(Arc::new(self.hashers.remove(0)));
            }

            // starting one consumer thread per (w,k)
            for i in 0..nidx {
                let table = Arc::clone(&tbl_refs[i]);
                let hasher = Arc::clone(&hsh_refs[i]);
                let mut char_rx = char_tx.add_rx();
                let w = self.w[i];
                let k = self.k[i];

                thread_handlers.push(thread::spawn(move || {
                    let mut window: XString<C> = XString::new(); // text window containing all kmers
                    let win_len = w + k;
                    let mut win_rks: MQueue<(TMmRank, usize)> = MQueue::new_min(); // window kmers ranks
                    let mut prev_mm_rk = 0 as TMmRank; // rank of previous window minimiser
                    let mut prev_right_rk = 0 as TMmRank; // rank of previous window rightmost kmer
                    let mut table = table.lock().unwrap();
                    let mut pos = 0;
                    loop {
                        //println!("loop consumer #{}", i);
                        match char_rx.recv() {
                            Ok(c) => {
                                // process char
                                // perpare window
                                if pos >= win_len {
                                    window.rotate_left(1);
                                    window[win_len - 1] = c;
                                } else {
                                    //read in first window
                                    window.push(c);
                                }
                                pos += 1;
                                if pos == k {
                                    let kmer_rk = hasher.hash(&window[window.len() - k..]);
                                    prev_right_rk = kmer_rk;
                                    prev_mm_rk = kmer_rk;
                                    let kmer_pos = pos - k;
                                    win_rks.push((kmer_rk, kmer_pos));
                                    // initial end minimisers are all indexed
                                    if !table.contains_key(&kmer_rk) {
                                        table.insert(kmer_rk, vec![offset + kmer_pos]);
                                    } else {
                                        table.get_mut(&kmer_rk).unwrap().push(offset + kmer_pos);
                                    }
                                } else if pos > k {
                                    // get previous windows minimiser
                                    // let (last_mm_rk, _last_mm_pos) = win_rks[i].xtr().unwrap().clone();
                                    // compute new last kmer rank and add it to the new window
                                    let kmer_rk = hasher
                                        .roll_hash(&window[window.len() - k - 1..], prev_right_rk);
                                    let kmer_pos = pos - k;
                                    prev_right_rk = kmer_rk;
                                    // dequeue the first kmer of previous window if it is full
                                    if pos > w + k - 1 {
                                        win_rks.pop();
                                    }
                                    // and add new kmer
                                    win_rks.push((kmer_rk, kmer_pos));
                                    // then get current window miminiser
                                    let cur_mm_rk = win_rks.xtr().unwrap().0;
                                    if w == 1 || prev_mm_rk != cur_mm_rk {
                                        // new minimiser. add all its occurrences
                                        let mm_pos: Vec<usize> =
                                            win_rks.xtr_iter().map(|(r, p)| offset + p).collect();
                                        if !table.contains_key(&cur_mm_rk) {
                                            table.insert(cur_mm_rk, mm_pos);
                                        } else {
                                            table.get_mut(&cur_mm_rk).unwrap().extend(mm_pos);
                                        }
                                        prev_mm_rk = cur_mm_rk;
                                    } else if cur_mm_rk == kmer_rk {
                                        // last kmer is a new occ of same old mm
                                        table.get_mut(&cur_mm_rk).unwrap().push(offset + kmer_pos);
                                    }
                                }
                                //println!("consumer #{} read  pos={}", i, pos);
                            }
                            Err(..) => {
                                // finished reading chars
                                // index end minimisers
                                while win_rks.len() > 1 {
                                    let (last_mm_rk, _last_mm_pos) = win_rks.xtr().unwrap().clone();
                                    win_rks.pop();
                                    let (cur_mm_rk, _cur_mm_pos) = win_rks.xtr().unwrap().clone();
                                    if last_mm_rk != cur_mm_rk {
                                        // new minimiser
                                        let mm_pos: Vec<usize> =
                                            win_rks.xtr_iter().map(|(r, p)| offset + p).collect();
                                        if !table.contains_key(&cur_mm_rk) {
                                            table.insert(cur_mm_rk, mm_pos);
                                        } else {
                                            table.get_mut(&cur_mm_rk).unwrap().extend(mm_pos);
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                    return pos;
                }));
            }

            // read in source stream
            while let Some(c) = s.get()? {
                char_tx.broadcast(c);
                strlen += 1;
            }
        } // the bus tx is dropped here. rx´s are "informed" that broadcast is over

        for h in thread_handlers {
            assert_eq!(strlen, h.join().expect("Thread failed to join."));
        }

        // restore MmIndex internal components
        while tbl_refs.len() > 0 {
            self.tables.push(
                Arc::try_unwrap(tbl_refs.remove(0))
                    .unwrap()
                    .into_inner()
                    .unwrap(),
            );
        }
        while hsh_refs.len() > 0 {
            self.hashers
                .push(Arc::try_unwrap(hsh_refs.remove(0)).unwrap());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::DNAAlphabet;
    use crate::xstream::XStrStream;
    use crate::xstring::XString;
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
        let dna_ab = DNAAlphabet::new_with_permutation(&letters);
        let mut ranker = vec![];
        ranker.push(KmerLexHasher::new(
            DNAAlphabet::new_with_permutation(&letters),
            k[0],
        ));
        println!("letters[0] = {0:?}", letters);
        //letters.rotate_left(1);
        println!("letters[1] = {0:?}", letters);
        ranker.push(KmerLexHasher::new(
            DNAAlphabet::new_with_permutation(&letters),
            k[1],
        ));
        //letters.rotate_left(1);
        println!("letters[2] = {0:?}", letters);
        ranker.push(KmerLexHasher::new(
            DNAAlphabet::new_with_permutation(&letters),
            k[2],
        ));
        let ranker_refs = [&ranker[0], &ranker[1], &ranker[2]];
        //                           0         1         2         3
        let mut mmindex = MmIndex::new(&w, &k, dna_ab);

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

    #[test]
    fn test_index_par_minimisers() {
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
        let dna_ab = DNAAlphabet::new_with_permutation(&letters);
        let mut ranker = vec![];
        ranker.push(KmerLexHasher::new(
            DNAAlphabet::new_with_permutation(&letters),
            k[0],
        ));
        println!("letters[0] = {0:?}", letters);
        //letters.rotate_left(1);
        println!("letters[1] = {0:?}", letters);
        ranker.push(KmerLexHasher::new(
            DNAAlphabet::new_with_permutation(&letters),
            k[1],
        ));
        //letters.rotate_left(1);
        println!("letters[2] = {0:?}", letters);
        ranker.push(KmerLexHasher::new(
            DNAAlphabet::new_with_permutation(&letters),
            k[2],
        ));
        let ranker_refs = [&ranker[0], &ranker[1], &ranker[2]];
        //                           0         1         2         3
        let mut mmindex = MmIndex::new(&w, &k, dna_ab);

        let src = XString::from("ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT".as_bytes());
        let mut stream = XStrStream::open(&src);
        mmindex
            .index_par(&mut stream)
            .expect("Error indexing stream");
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
