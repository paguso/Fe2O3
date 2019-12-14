use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::xstring::XString;

pub struct FastaReader<R>
where
    R: Read,
{
    reader: BufReader<R>,
}

impl FastaReader<File> {
    pub fn new_from_path<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        Ok(FastaReader {
            reader: BufReader::new(File::open(path)?),
        })
    }
}

impl<R> FastaReader<R>
where
    R: Read + Seek,
{
    pub fn new(src: R) -> Result<Self, std::io::Error> {
        Ok(FastaReader {
            reader: BufReader::new(src),
        })
    }

    /// Reads the next FASTA record as a `(desc, seq)` pair where
    /// * `desc` is a String with the record description line without the starting `>`
    /// * `seq` is the actual sequence as a XString<u8>
    /// EOL chars are not included in `desc` or `seq`
    pub fn next_as_xstring(&mut self) -> Result<Option<(String, XString<u8>)>, std::io::Error> {
        // read next id
        let mut desc = String::new();
        let mut n = self.reader.read_line(&mut desc)?;
        if n == 0 {
            return Ok(None);
        }
        assert_eq!(desc.as_bytes()[0], '>' as u8);
        desc.pop();
        desc.remove(0);
        let mut seq: Vec<u8> = Vec::new();
        loop {
            n = self.reader.read_until(0xA, &mut seq)?;
            if n == 0 {
                // EOF
                break;
            } else if seq[seq.len() - n] == '>' as u8 {
                // read description of next sequence. put it back
                seq.truncate(seq.len() - n);
                self.reader.seek(SeekFrom::Current(-(n as i64)))?;
                break;
            } else {
                // read one more line. trim EOL.
                assert_eq!(seq.pop(), Some(0xA));
            }
        }
        Ok(Some((desc, XString::from(seq))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufWriter, Write};

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

        let mut reader = FastaReader::new_from_path(&filename)
            .expect(&format!("Unable to open file {}", filename));
        let mut nseq = 0;
        while let Some((desc, s)) = reader
            .next_as_xstring()
            .expect("Unable to read from fasta file")
        {
            println!("fasta description line = {}\n sequence={1:?}", desc, s);
            nseq += 1;
        }
        assert_eq!(nseq, 3);
        file_teardown(&filename);
    }
}
