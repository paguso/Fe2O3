use std::cmp;
use crate::xstring::XString;
use crate::xstring::XStrRanker;
use crate::xstream::XStream;
use crate::alphabet::Alphabet;


fn find_minimisers<C>(s: impl XStream<CharType=C>, w: usize, k:usize, ranker: &impl XStrRanker<CharType=C> ) -> Option<Vec<usize>> 
where C : Copy + Default + Eq
{
    let mut window = XString::repeat(w+k-1, C::default());
    let mut wlen = 0;

    wlen = s.read(&mut window).unwrap();
    if wlen < k || w == 0 {
        // no kmer
        return None; 
    }
    window.truncate(wlen);

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
    for j in 0..=cmp::min(w-1, wlen-k) {
        pos_rank = ranker.rank(&window[j..j+k]);
        buf.push(pos_rank);
        if pos_rank <= last_min_rank { // (!) <=
            last_min_rank = pos_rank;
            last_min_pos = j;
        }      
    }
    for j in 0..=cmp::min(w-1, wlen-k) {
        if buf[j] == last_min_rank {
            occ.push(j);
        }
    }
    // process subsequent windows, if any
    while true {
        if s.eos().unwrap() {
            window.remove(0);
            wlen -= 1;
        }
        else {
            let c = s.get().unwrap();
            assert!(!c.is_none());
            window.rotate_left(1);
            window[wlen-1] = c.unwrap();
        }




    if n > (w+k-1) {
        assert_eq!(buf.len(), w);
        // current window starts at position i
        for i in 1..=n-(w+k-1) {
            println!("Processing window at position {}",i);
            // current window differs from previous by last kmer only
            // process last kmer of the window
            pos_rank = ranker.rank(&s[i+w-1..i+w-1+k]);
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
    }
    Some(occ)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use crate::alphabet::DNAAlphabet;
    use crate::xstring::XString;
    use crate::xstring::XStrLexRanker;

    #[test]
    fn test_find_minimisers() {
        let dna_ab = DNAAlphabet::new();
        let w = 4;
        let k = 5;
        let ranker = XStrLexRanker::new(Rc::new(dna_ab));
        let rs = String::from("acgtacgtacgtacgtacgtacgtacgtacgtacgtacgt");
        let mut s:XString<char> = XString::new();
        for c in rs.chars() {
            s.push(c);
        }
        match find_minimisers(&s, w, k, &ranker) {
            Some(occ) => {
                for j in &occ {
                    assert_eq!(&s[..k], &s[*j..*j+k]);
                }
            },
            None => assert!(false, "No minimisers found. Should have found some."),
        }
    }
}
