use crate::alphabet::{Alphabet, Character};
use crate::mqueue::MQueue;
use crate::xstream::XStream;
use crate::xstring::XStrRanker;
use crate::xstring::XString;
use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::cmp::{min, max};


fn find_minimisers<C>(
    s: &mut impl XStream<CharType = C>,
    w: usize,
    k: usize,
    ranker: &impl XStrRanker<CharType = C>,
) -> Option<Vec<usize>>
where
    C: Character,
{
    let mut window = XString::new();
    let mut wlen = 0;

    while wlen <= w + k - 1 {
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
        wscores.push((ranker.rank(&window[pos..pos + k]), pos));
    }

    let mut minimisers: Vec<usize> = vec![];
    let mut wmin;
    pos = wlen - k + 1;
    while wlen >= k {
        wmin = wscores.xtr().unwrap();
        minimisers.push(wmin.1);
        match s.get() {
            Ok(Some(c)) => {
                window.rotate_left(1);
                window[k - 1] = c;
                wscores.pop();
                wscores.push((ranker.rank(&window[wlen - k..]), pos));
                pos += 1;
            }
            _ => {
                window.remove(0);
                wlen -= 1;
            }
        }
    }
    Some(minimisers)
}


fn index_minimisers<C>(src: &mut impl XStream<CharType=C>, w:usize, k_vals: &[usize], ranker: &impl XStrRanker<CharType=C>) -> Result<HashMap<XString<C>, Vec<usize>>, io::Error> 
where 
    C: Character + Hash, 
{
    let mut index: HashMap<XString<C>, Vec<usize>> = HashMap::new();
    
    if k_vals.len()==0 || w==0 {
        return  Ok(index);
    }

    let mut sorted_k = vec![0usize; k_vals.len()];
    sorted_k.copy_from_slice(k_vals);
    sorted_k.sort();
    let k_count = sorted_k.len();
    let k_min = sorted_k[0];
    let k_max = sorted_k[k_count-1];

    let mut window = XString::new();
    let mut wlen = 0;    

    while wlen <= w + k_max - 1 {
        match src.get()? {
            Some(c) => {
                window.push(c);
                wlen += 1;
            }
            None => break,
        }
    }
    if wlen < k_min {
        return Ok(index);
    }

    let mut wscores: Vec<MQueue<(u64, usize)>> = vec![MQueue::new_min(); k_count];
    // process first window
    let mut pos: usize;
    for pos in 0..min(w, wlen-k_min+1) {
        for (i,k) in sorted_k.iter().enumerate() {
            if pos + k < wlen {
                wscores[i].push((ranker.rank(&window[pos..pos + k]), pos));
            }
        }
    }

    let mut w_end = wlen;
    while true {
        for (i,k) in sorted_k.iter().enumerate()  {
            if !wscores[i].is_empty() {
                let wmin = wscores[i].xtr().unwrap();
                let kmer = XString::from(&window[wmin.1..wmin.1+k]);
                match index.get_mut(&kmer) {
                    Some(occ) => {occ.push(wmin.1);},
                    None => {index.insert(kmer, vec![wmin.1]);}
                }
            }
        }       
        match src.get()? {
            Some(c) => {
                window.rotate_left(1);
                window[wlen - 1] = c;
                for (i,k) in sorted_k.iter().enumerate() {
                    if !wscores[i].is_empty() {
                        wscores[i].pop();
                        wscores[i].push((ranker.rank(&window[wlen - k..]), pos));
                    }
                }
            }
            None => {
                window.remove(0);
                wlen -= 1;
                if wlen < k_min {
                    break;
                }
            }
        }
    }

    Ok(index)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::DNAAlphabet;
    use crate::xstream::XStrStream;
    use crate::xstring::{XStrLexRanker, XString};
    use std::rc::Rc;

    #[test]
    fn test_find_minimisers() {
        let dna_ab = DNAAlphabet::new();
        let w = 4;
        let k = 5;
        let ranker = XStrLexRanker::new(Rc::new(dna_ab));
        let mut src = XString::from("acgtacgtacgtacgtacgtacgtacgtacgtacgtacgt".as_bytes());
        let mut stream = XStrStream::open(src);
        let minimisers = find_minimisers(&mut stream, w, k, &ranker).unwrap();
        src = stream.close();
        for j in &minimisers {
            println!("minimiser found at position {}", j);
            assert_eq!(&src[..k], &src[*j..*j + k]);
        }
    }
}
