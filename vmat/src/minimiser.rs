use crate::alphabet::{Alphabet, Character};
use crate::mqueue::MQueue;
use crate::xstream::XStream;
use crate::xstring::XStrRanker;
use crate::xstring::XString;
use std::collections::HashMap;
use std::hash::Hash;


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


fn index_minimisers<C>(src: &mut impl XStream<CharType=C>, w:usize, k_vals: &[usize], ranker: &impl XStrRanker<CharType=C>) -> Result<HashMap<XString<C>, Vec<usize>>, std::io::Error> 
where 
    C: Character + Hash, 
{
    let mut sorted_k = vec![k_vals.len(); 0usize];
    sorted_k.copy_from_slice(k_vals);
    sorted_k.sort();

    let mut index: HashMap<XString<C>, Vec<usize>> = HashMap::new();

    let mut window = XString::new();
    let mut wlen = 0;

    

    while wlen <= w + k - 1 {
        match src.get() {
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
