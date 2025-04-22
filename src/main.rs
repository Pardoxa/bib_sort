use std::{
    collections::BTreeSet, 
    fs::File, 
    io::{stdout, BufRead, BufReader, BufWriter, Write}, 
    path::PathBuf, 
    process::exit
};
use clap::Parser;

mod sort_by_author;

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
/// Note: If you want to overwrite the bibfile: Do NOT pipe into it. 
/// Commands like bib_sort literature.bib > literature.bib will DELETE the literature.bib file 
/// before the program reads it, which means it is essentially just creating an empty file.
/// 
/// INSTEAD: You can safely use bib_sort literature.bib -o literature.bib as this will first parse
/// the bibfile and then overwrite it with the sorted file only if no errors were detected.
#[derive(Parser)]
pub struct Opts{
    /// Path to the current bib file
    bib_path: PathBuf,

    #[arg(long, short)]
    /// Make sorting case sensitive
    case_sensitive: bool,

    #[arg(long, short)]
    /// If this option is used, the output will be written to the specified file instead of printed to stdout
    out: Option<PathBuf>,

    /// By default the program searches for duplicate keys and gives an error message upon detection.
    /// This flag can be used to turn this off. [alias: --ndd]
    #[arg(long, short, alias="ndd")]
    no_duplicate_detection: bool,

    /// By default the program will not allow bib items that do not have a key,
    /// as those items cannot be cited and are likely a mistake.
    /// However, some people like to have empty items for @article, @book etc
    /// in the beginning of the file. This option will allow empty keys.
    /// Also: Empty keys will be exempt from the duplicate detection.
    /// [alias: --aek]
    #[arg(long, alias="aek")]
    allow_empty_keys: bool,

    /// Additionally use the doi of the bib items to search for duplicates.
    /// [alias: --add]
    #[arg(long, alias="add")]
    allow_doi_duplicates: bool,

    /// For the doi items: This option ignores items with empty doi like "doi = {},"
    /// Note: Items without "doi = " (arbitrary amount of spaces) are always
    /// ignored for doi duplicate detection.
    /// [alias: --aed]
    #[arg(long, alias="aed")]
    allow_empty_doi: bool,

    /// Parses the "author = " part, truncates it such that it only contains 
    /// the first author and uses that to sort. This sorting depends on the ordering of 
    /// first and last name in the bib entry
    /// [alias: --sba]
    #[arg(long, alias="sbfaf", conflicts_with = "sort_by_first_author_first_name")]
    sort_by_first_author_field: bool,

    /// Parses the "author = " part, truncates it such that it only contains 
    /// the first author. Then it tries to put the first name of the 
    /// author in front of the last name, if the order was reversed.
    /// This is then used for sorting.
    /// [alias: --sbfafn]
    #[arg(long, alias="sbfafn")]
    sort_by_first_author_first_name: bool
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

    let reader = File::open(opts.bib_path)
        .expect("Cannot open bibfile");
    let buf_reader = BufReader::new(reader);

    let lines = buf_reader
        .lines()
        .map(|entry| entry.expect("Error reading line - your bibfile needs to be encoded with UTF8"));
    let mut line_iter_helper = LineIterHelper::new(lines);

    let mut entries = Vec::new();

    let case_fn = if opts.case_sensitive {
        str::to_owned
    } else {
        str::to_lowercase
    };

    // regex for where bibentries start
    let entry_start = r"@.*\{";
    let re = regex::Regex::new(entry_start).unwrap();
    let id_regex = regex::Regex::new(r"[^,\s]+")
        .unwrap();

    while let Some(line) = line_iter_helper.next() {
        let no_leading_whitespace = line.trim_start();
        if no_leading_whitespace.is_empty(){
            continue;
        }
        if !no_leading_whitespace.starts_with('@'){
            panic!("Missmatched brackets? Encountered line outside bib items that does not start with @, i.e., that does not start a new bib item. Line was {line}");
        }

        let id = match re.find(no_leading_whitespace)
        {
            Some(m) => {
                let entry_line = &no_leading_whitespace[m.end()..];
                match id_regex.find(entry_line) {
                    None => {
                        if opts.allow_empty_keys{
                            "".to_owned()
                        } else {
                            panic!("Cannot find key in line: {line}")
                        }
                    },
                    Some(id_match) => {
                        case_fn(id_match.as_str())
                    }
                }
            },
            None => {
                panic!("Line without whitespaces starts with @ - but cannot parse - Missing {{?");
            }
        };

        let mut bracket_counter = BracketCounter::default();
        let mut content = bracket_counter.count_brackets_return_content(
            no_leading_whitespace,
            &mut line_iter_helper
        );
        

        while !bracket_counter.equal_brackets() {
            let next_line = line_iter_helper.next()
                .expect("Unexpected end of file - did you forget to close a bracket?");
            
            content.push('\n');
            content.push_str(
                &bracket_counter.count_brackets_return_content(
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
    // Dropping line_iter_helper such that 
    // the file handle of the reader is dropped as well,
    // so there is no issue writing to the same file 
    // if the user wants to
    drop(line_iter_helper);


    entries.sort_by_cached_key(|entry| entry.id.clone());
    if !opts.no_duplicate_detection{
        let mut detected_duplicates = false;
        entries.windows(2)
            .for_each(
                |slice|
                {
                    // Either there are no empty keys, or they were explicitly allowed 
                    // and in that case they are exempt from duplication detection
                    if !slice[0].id.is_empty() && slice[0].id == slice[1].id {
                        detected_duplicates = true;
                        eprintln!("Duplicate key: {}", slice[0].id);
                    }
                }
            );

        if !opts.allow_doi_duplicates {
            let mut doi_set = BTreeSet::new();

            let doi_position_regex = regex::Regex::new(r"(?i)\bdoi\s*=\s*")
                .unwrap();
            let doi_regex = regex::Regex::new(r"10\.[\)\(\.\w/\-:]+")
                .unwrap();

            for BibEntry { content, id } in entries.iter()
            {
                // ignore items with empty id
                if id.is_empty() {
                    continue;
                }

                // check if it contains something like "doi = " (case insensitive)
                if let Some(doi_pos_match) = doi_position_regex.find(content)
                {
                    let mut str_containing_doi_next = &content[doi_pos_match.end()..];
                    // doi comes before "," if there is any ","
                    // - If there is no Doi given in the doi field, this makes sure the regex does not try 
                    // to find the Doi in other parts of the bibitem
                    if let Some((doi_part, ..)) = str_containing_doi_next.split_once(',')
                    {
                        str_containing_doi_next = doi_part;
                    }

                    match doi_regex.find(str_containing_doi_next)
                    {
                        None => {
                            if opts.allow_empty_doi {
                                continue;
                            }
                            panic!(
                                "Error - cannot parse DOI in item with key {id}, even though it contains\n'{}'\nFull content of item is\n{content}", 
                                doi_pos_match.as_str()
                            );
                        },
                        Some(doi) => {
                            let item_was_not_previously_inserted = doi_set.insert(doi.as_str());
                            if !item_was_not_previously_inserted {
                                detected_duplicates = true;
                                eprintln!("Duplicate DOI: {}", doi.as_str());
                            }
                        }
                    }

                }
            }
        }
        
        if detected_duplicates {
            panic!(
                "The program detected that your file contains either at least one duplicate key or at least one duplicate doi (see previous error messages)!\n\
                 Please have a look at your file and fix this error before trying again \
                 or have a look at the options starting with --allow\n\
                 You can find all options by using --help\n\
                 Aborted writing anything."
            )
        }
    }

    if opts.sort_by_first_author_field{
        let case_fn = get_string_case_fn(opts.case_sensitive);

        sort_by_author::sort_by_first_author_field(
            &mut entries,
            case_fn
        );
    }

    if opts.sort_by_first_author_first_name {
        let case_fn = get_string_case_fn(opts.case_sensitive);

        sort_by_author::sort_by_first_author_first_name(
            &mut entries, 
            case_fn
        );
    }
    
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
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                exit(0);
            } else {
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

    fn count_brackets_return_content<I>(
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
                    } else if self.close > self.open {
                        panic!("Bracket was closed before it was opened! Mismatched bracket error in: {s}");
                    }

                },
                _ => ()
            }
        }
        content
    }
}


pub fn get_string_case_fn(case_sensitive: bool) -> impl Fn(String) -> String
{
    if case_sensitive {
        |string| string
    } else {
        |string: String| {
            string.to_lowercase()
        }
    }
}