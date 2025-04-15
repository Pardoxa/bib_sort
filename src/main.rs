use std::{
    fs::File, 
    io::{stdout, BufRead, BufReader, BufWriter, Write}, 
    path::PathBuf
};
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
#[derive(Parser)]
pub struct Opts{
    /// Path to the current bib file
    bib_path: PathBuf,

    #[arg(long, short)]
    /// Make sorting case sensitive
    case_sensitive: bool,

    #[arg(long, short)]
    /// If this option is used, the output will be written to the specified file instead of printed to stdout
    out: Option<PathBuf>
}

pub struct LineIterHelper<I>{
    pub line_iter: I,
    pub leftover: Option<String>
}

impl<I> Iterator for LineIterHelper<I>
where I: Iterator<Item=String>
{
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.leftover
            .take()
            .or_else(|| self.line_iter.next())
    }
}

impl<I> LineIterHelper<I>
    where I: Iterator<Item = String>
{
    pub fn new(line_iter: I) -> Self
    {
        Self {
            leftover: None,
            line_iter
        }
    }
}


fn main() {

    let opts = Opts::parse();

    let reader = File::open(opts.bib_path.as_path())
        .expect("Cannot open bibfile");
    let buf_reader = BufReader::new(reader);

    let lines = buf_reader
        .lines()
        .map(|entry| entry.expect("Error reading line - your bibfile needs to be encoded with UTF8"));
    let mut line_iter_helper = LineIterHelper::new(lines);

    let mut entries = Vec::new();

    while let Some(line) = line_iter_helper.next() {
        let no_leading_whitespace = line.trim_start();
        if no_leading_whitespace.is_empty(){
            continue;
        }
        if !no_leading_whitespace.starts_with('@'){
            panic!("Missmatched brackets? Encountered line outside bib items that does not start with @, i.e., that does not start a new bib item. Line was {line}");
        }

        let entry_start = r"@.*\{";
        let re = regex::Regex::new(entry_start).unwrap();

        let id = match re.find(no_leading_whitespace)
        {
            Some(m) => {
                let trimmed = no_leading_whitespace[m.end()..]
                    .trim_start();
                
                if opts.case_sensitive{
                    trimmed.to_owned()
                } else {
                    trimmed.to_lowercase()
                }
                
            },
            None => {
                panic!("Line without whitespaces starts with @ - but cannot parse - Missing {{?");
            }
        };

        let mut bracket_counter = BracketCounter::default();
        let mut content = bracket_counter.count_brackets_return_leftover(
            no_leading_whitespace,
            &mut line_iter_helper
        );
        

        while !bracket_counter.equal_brackets() {
            let next_line = line_iter_helper.next()
                .expect("Unexpected end of file - did you forget to close a bracket?");
            
            content.push('\n');
            content.push_str(
                &bracket_counter.count_brackets_return_leftover(
                    &next_line,
                    &mut line_iter_helper
                )
            );
        }
        let bib_entry = BibEntry{
            id,
            content
        };

        entries.push(bib_entry);
    }
    entries.sort_by_cached_key(|entry| entry.id.clone());
    
    match opts.out{
        None => {
            let out = stdout();
            write_entries(entries, out);
        },
        Some(out_path) => {
            let out_file = File::create(out_path)
                .expect("Unable to create file");
            let out = BufWriter::new(out_file);
            write_entries(entries, out);
        } 
    }


}

pub fn write_entries<W: Write>(entries: Vec<BibEntry>, mut out: W){
    for entry in entries{
        let io_result = writeln!(out, "{}\n", entry.content);
        if let Err(e) = io_result{
            // ignore broken pipes
            if !matches!(e.kind(), std::io::ErrorKind::BrokenPipe) {
                panic!("{e}");
            }
        }
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

    fn count_brackets_return_leftover<I>(
        &mut self, 
        s: &str, 
        line_iter_helper: &mut LineIterHelper<I>
    ) -> String
    {
        let mut char_iter = s.chars();
        let mut content = String::new();
        for c in &mut char_iter{
            content.push(c);
            match c {
                '{' => {
                    self.open += 1;
                },
                '}' => {
                    self.close += 1;
                    if self.equal_brackets() {
                        let leftover: String = char_iter.collect();
                        if !leftover.is_empty(){
                            line_iter_helper.leftover = Some(leftover);
                        }
                        return content;
                    }

                },
                _ => ()
            }
        }
        content
    }
}
