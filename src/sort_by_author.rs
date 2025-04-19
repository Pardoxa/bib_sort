use std::process::exit;

use crate::BibEntry;
use regex::Regex;
use lazy_static::lazy_static;


lazy_static! {
    static ref AUTHOR_POS_REGEX: Regex = {
        Regex::new(r"(?i)\bauthor\s*=\s*")
            .unwrap()
    };

    static ref AUTHOR_REGEX: Regex = {
        Regex::new(r"\w+\s*,?\s*\w*")
            .unwrap()
    };

    static ref AND: Regex = {
        Regex::new(r"(?i)\band\b")
            .unwrap()
    };
}



pub fn author_from_content(content: &str) -> &str
{
    match AUTHOR_POS_REGEX.find(content){
        None => {
            ""
        },
        Some(author_pos_match) => {
            let mut author_field_next = &content[author_pos_match.end()..];

            author_field_next = field_content(author_field_next);

            //if let Some(and_match) =  AND.find(author_field_next) {
            //    author_field_next = &author_field_next[..and_match.start()];
            //}

            author_field_next

        }
    }
}


pub fn sort_by_author(to_sort: &mut [BibEntry])
{
    to_sort.iter()
        .for_each(
            |bib_entry|
            {
                let author = author_from_content(&bib_entry.content);
                println!("{author}");
            }
        );
    exit(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketOrQuote{
    None,
    OpenBracket(u32),
    SingleQuote,
    DoubleQuote
}


pub fn field_content(mut field: &str) -> &str
{
    let mut bracket_or_quote = BracketOrQuote::None;
    let mut start = 0;
    let mut iter = field.char_indices();
    
    for (index, char) in &mut iter{
        match &mut bracket_or_quote {
            BracketOrQuote::None => {
                start = index;
                match char {
                    '{' => {
                        bracket_or_quote = BracketOrQuote::OpenBracket(1);
                    },
                    '\'' => {
                        bracket_or_quote = BracketOrQuote::SingleQuote;
                    },
                    '"' => {
                        bracket_or_quote = BracketOrQuote::DoubleQuote;
                    },
                    _ => continue
                }
            },
            BracketOrQuote::OpenBracket(num) => {
                match char {
                    '{' => *num += 1,
                    '}' => *num -= 1,
                    _ => continue
                }
                if *num == 0 {
                    field = match iter.next(){
                        Some((idx, _)) => {
                            &field[start..idx]
                        },
                        None => {
                            &field[start..]
                        }
                    };
                    break;
                }
            },
            BracketOrQuote::SingleQuote => {
                if char == '\'' {
                    field = match iter.next(){
                        Some((idx, _)) => {
                            &field[start..idx]
                        },
                        None => {
                            &field[start..]
                        }
                    };
                    break;
                }
            },
            BracketOrQuote::DoubleQuote => {
                if char == '"' {
                    field = match iter.next(){
                        Some((idx, _)) => {
                            &field[start..idx]
                        },
                        None => {
                            &field[start..]
                        }
                    };
                    break;
                }
            }
        }
    }

    field
}