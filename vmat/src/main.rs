use std::rc::Rc;

use clap::{App, Arg, SubCommand};

use vmat::dna::DNAAlphabet;
use vmat::dna::DNAHasher;
use vmat::minimiser;
use vmat::xstream::XStrFileReader;
use vmat::xstring::XString;

fn index(input_filename: &str, output_filename: &str, w: &[usize], k: &[usize]) {
    println!("Indexing {} to {}", input_filename, output_filename);

    if (w.len() != k.len()) {
        println!(
            "Error: The quantities of window sizes (-w) and minimiser lengths (-k) must agree."
        );
        println!("       Window sizes: {0:?}", w);
        println!("       Minimiser lengths: {0:?}", k);
        std::process::exit(1);
    }

    let mut reader: XStrFileReader<u8> =
        XStrFileReader::new(input_filename).expect("Cannot open input file");
    let m = w.len();

    let dna_ab = DNAAlphabet::new();
    let mut letters = [
        DNAAlphabet::a,
        DNAAlphabet::c,
        DNAAlphabet::g,
        DNAAlphabet::t,
    ];
    let mut ranker = vec![];
    for i in 0..m {
        ranker.push(DNAHasher::new(Rc::new(DNAAlphabet::new_with_permutation(
            &letters,
        ))));
        letters.rotate_left(1);
    }
    let ranker_refs = [&ranker[0], &ranker[1], &ranker[2]];

    minimiser::index_minimisers(&mut reader, w, k, &ranker_refs);

    println!("Done");
}

fn main() {
    let matches = App::new("VMAT - Variable Minimiser Alignment Tool")
        .version("0.1")
        .author("Paulo Fonseca (paguso@cin.ufpe.br)")
        .about("Variable Minimiser Alignment Tool")
        .subcommand(
            SubCommand::with_name("index")
                .about("Creates the index")
                //.version("1.3")
                //.author("Someone E. <someone_else@other.com>")
                .arg(
                    Arg::with_name("input")
                        .index(1)
                        .short("i")
                        .help("input file")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .help("Index output file")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("window")
                        .short("w")
                        .help("Window sizes = # of kmers")
                        .value_name("SIZES")
                        .required(false)
                        .requires("kmer")
                        .takes_value(true)
                        .default_value("20")
                        .min_values(1),
                )
                .arg(
                    Arg::with_name("kmer")
                        .short("k")
                        .help("Minimiser lengths")
                        .value_name("SIZES")
                        .required(false)
                        .requires("window")
                        .takes_value(true)
                        .default_value("10")
                        .min_values(1),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("index") {
        let input_filename = matches.value_of("input").unwrap();
        let output_filename = match matches.value_of("output") {
            Some(filename) => String::from(filename),
            None => format!("{}.idx", input_filename),
        };
        let w: Vec<usize> = matches
            .values_of("window")
            .unwrap()
            .map(|x| usize::from_str_radix(x, 10).unwrap())
            .collect();
        let k: Vec<usize> = matches
            .values_of("kmer")
            .unwrap()
            .map(|x| usize::from_str_radix(x, 10).unwrap())
            .collect();
        println!("Windows={0:?}", w);
        println!("Kmer={0:?}", k);
        index(input_filename, output_filename.as_str(), &w, &k);
    }
}
