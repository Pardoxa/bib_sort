use std::{fs::File, io::{BufRead, BufReader}};


fn main() {
    let reader = File::open("alex_refs.bib").unwrap();
    let buf_reader = BufReader::new(reader);

    let mut lines = buf_reader.lines()
        .map(|entry| entry.unwrap())
        .peekable();

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
                eprintln!("None?");
                continue;
            }
        };

        let mut open_brackets = count_left_braces(no_leading_whitespace);
        let mut closed_brackets = count_right_braces(no_leading_whitespace);


        let mut content = line;

        while closed_brackets != open_brackets {
            let next_line = lines.next().unwrap();
            open_brackets += count_left_braces(&next_line);
            closed_brackets += count_right_braces(&next_line);
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

fn count_left_braces(s: &str) -> usize {
    s.chars().filter(|&c| c == '{').count()
}

fn count_right_braces(s: &str) -> usize {
    s.chars().filter(|&c| c == '}').count()
}