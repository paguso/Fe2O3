use std::cmp;
use crate::xstring::XString;
use crate::xstring::XStrRanker;
use crate::xstream::XStream;
use crate::alphabet::Alphabet;
use crate::mqueue::MQueue;


fn find_minimisers<C>(s: &mut impl XStream<CharType=C>, w: usize, k:usize, ranker: &impl XStrRanker<CharType=C> ) -> Option<Vec<usize>> 
where C : Copy + Default + Eq
{
    
    let mut window = XString::repeat(w+k-1, C::default());
    let mut wlen;

    wlen = s.read(&mut window).unwrap();
    if wlen < k || w == 0 {
        // no kmer
        return None; 
    }
    window.truncate(wlen);

    let mut wscores: MQueue<(u64,usize)> = MQueue::new_min();
    // process first window
    let mut pos:usize;
    for pos in 0..wlen-k+1 {
        wscores.push( (ranker.rank(&window[pos..pos+k]), pos) );
    }

    let mut minimisers: Vec<usize> = vec![];
    let mut wmin;
    pos = wlen-k+1;
    while wlen >= k {
        wmin = wscores.xtr().unwrap();
        minimisers.push(wmin.1);
        match s.get()  {
            Ok(Some(c)) => {
                window.rotate_left(1);
                window[k-1] = c;
                wscores.push( (ranker.rank(&window[wlen-k..]) , pos) );
                pos += 1;
            },
            _ => {
                window.remove(0);
                wlen -= 1;
            }
        }
    }
    Some(minimisers)   
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
        /*
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
        */
    }
}
