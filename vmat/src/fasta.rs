use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::rc::Rc;

use crate::alphabet::Alphabet;
use crate::xstring::XString;

pub struct FastaScanner {
    reader: BufReader<File>,
}

impl FastaScanner {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let ret = FastaScanner {
            reader: BufReader::new(File::open(path)?),
        };
        Ok(ret)
    }

    pub fn next_as_xstring(&mut self) -> Result<Option<(String, XString<u8>)>, std::io::Error> {
        // read next id
        let mut id = String::new();
        let mut n = self.reader.read_line(&mut id)?;
        if n == 0 {
            return Ok(None);
        }
        assert_eq!(id.as_bytes()[0], '>' as u8);
        id.pop();
        id.remove(0);
        let mut seq: Vec<u8> = Vec::new();
        loop {
            n = self.reader.read_until(0xA, &mut seq)?;
            println!("seq={0:?}", seq);
            if n==0 { // EOF
                break; 
            }
            else if seq[seq.len() - n] == '>' as u8 { // read description of next sequence. put it back
                seq.truncate(seq.len() - n);
                self.reader.seek(SeekFrom::Current(-(n as i64)))?;
                break;
            } else { // read one more line. trim EOL.
                assert_eq!(seq.pop(), Some(0xA));
            }
        }
        Ok(Some((id, XString::from(seq))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Write, BufWriter};

    const FASTA_FILE: &'static [u8] = b">id1 desc
AAAAAAAAAA
AAAAAAAAAA
AAAAAAAAAA
AAAAAAAAAA
AAAAA
>id2
CCCCCCCCCCCCCCCCCCCC
CCCCCCCCCCCCCCCCCCCC
CCCCCCCCCCCCCCCCCCCC
CCCCCCCCCCCCCCCCCCCC
>id3 lots of Gs
GGGGGGGGGG
GGGGGGGGGG
GGGGGGGGGG
GGGGGGGGGG
GGGGGGGGGG
GGGGGGGGGG
";

    fn file_setup(filename: &str) {
        let f = File::create(filename).expect("Unabe to create the file");
        let mut writer = BufWriter::new(f);
        writer.write(&FASTA_FILE);
    }

    fn file_teardown(filename: &str) {
        std::fs::remove_file(filename).expect("Unable to delete file");
    }

    #[test]
    fn read_fasta() {
        let filename = "read_fasta.fas";
        file_setup(&filename);

        let mut scanner = FastaScanner::open(&filename).expect(&format!("Unable to open file {}", filename));
        let mut nseq = 0;
        while let Some((desc, s)) = scanner.next_as_xstring().expect("Unable to read from fasta file") {
            println!("fasta description line = {}\n sequence={1:?}", desc, s);
            nseq +=1;
        }
        assert_eq!(nseq, 3);
        file_teardown(&filename);
    }
}
