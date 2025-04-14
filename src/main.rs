use std::{fs::File, io::{BufRead, BufReader}, path::PathBuf};
use clap::Parser;

/// Created by Yannick Feld
/// 
/// The program is intended to sort bibfiles by the key.
/// For example
/// 
/// @article{boers2019,
///     author = {N. Boers AND B. Goswami AND A. Rheinwalt AND B. Bookhagen AND B. Hoskins AND J. Kurths},
///     title = {Complex networks reveal global pattern of extreme-rainfall teleconnections},
///     journal = {Nature},
///     year = 2019,
///     volume = {566},
///     pages = {373-377},
///     doi = {10.1038/s41586-018-0872-x}
/// }
/// 
/// 
/// Here the key is boers2019
/// 
/// Note: A bib item is NOT allowed to start on the same line where another bib item ended, meaning there needs to be 
/// at least one newline character at the end of each bib item (except the last one).
/// Otherwise the output may be garbage
/// 
#[derive(Parser)]
pub struct Opts{
    /// Path to the current bib file
    bib_path: PathBuf
}


fn main() {

    let opts = Opts::parse();

    let reader = File::open(opts.bib_path.as_path())
        .expect("Cannot open bibfile");
    let buf_reader = BufReader::new(reader);

    let mut lines = buf_reader
        .lines()
        .map(|entry| entry.expect("Error reading line - your bibfile needs to be encoded with UTF8"));

    let mut entrys = Vec::new();

    while let Some(line) = lines.next() {
        let no_leading_whitespace = line.trim_start();
        if !no_leading_whitespace.starts_with('@'){
            continue;
        }

        let entry_start = r"@.*\{";
        let re = regex::Regex::new(entry_start).unwrap();

        let id = match re.find(no_leading_whitespace)
        {
            Some(m) => {
                no_leading_whitespace[m.end()..].trim_start().to_lowercase()
            },
            None => {
                panic!("Line without whitespaces starts with @ - but cannot parse - error");
            }
        };

        let mut bracket_counter = BracketCounter::default();
        bracket_counter.count_brackets(no_leading_whitespace);

        let mut content = line;

        while !bracket_counter.equal_brackets() {
            let next_line = lines.next()
                .expect("Unexpected end of file - did you forget to close a bracket?");
            bracket_counter.count_brackets(&next_line);
            content.push('\n');
            content.push_str(&next_line);
        }
        let bib_entry = BibEntry{
            id,
            content
        };

        entrys.push(bib_entry);
    }
    entrys.sort_by_cached_key(|entry| entry.id.clone());
    

    for entry in entrys{
        println!("{}", entry.content);
        println!();
    }
}

#[derive(Debug)]
pub struct BibEntry{
    pub id: String,
    pub content: String
}


#[derive(Clone, Copy, Debug, Default)]
pub struct BracketCounter{
    open: u32,
    close: u32
}

impl BracketCounter{
    pub fn equal_brackets(&self) -> bool 
    {
        self.open == self.close
    }

    fn count_brackets(&mut self, s: &str)
    {
        s.chars()
            .for_each(
                |c|
                {
                    match c {
                        '{' => {
                            self.open += 1;
                        },
                        '}' => {
                            self.close += 1;
                        },
                        _ => ()
                    }
                }
            );
    }
}
