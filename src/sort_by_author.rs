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



pub fn first_author_from_content(content: &str) -> String
{
    match AUTHOR_POS_REGEX.find(content){
        None => {
            "".to_owned()
        },
        Some(author_pos_match) => {
            let mut author_field_next = &content[author_pos_match.end()..];

            author_field_next = first_author_field_content(author_field_next);

            if let Some(and_match) =  AND.find(author_field_next) {
                author_field_next = &author_field_next[..and_match.start()];
            }

            clean_string(author_field_next)

        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode{
    NotEscaped,
    Escaped   
}

pub fn clean_string(s: &str) -> String
{
    let mut string = String::with_capacity(s.len());
    let mut mode = Mode::NotEscaped;
    for c in s.chars()
    {
        match c {
            char if mode == Mode::Escaped => {
                // If mode is escaped: next char is always included
                string.push(char);
                mode = Mode::NotEscaped;
            },
            '\'' | '\"' | '{' | '}' if mode == Mode::NotEscaped => {
                // ignoring those if not escaped
                continue;
            },
            '\\' => {
                string.push('\\');
                mode = Mode::Escaped;
            },
            _ => {
                string.push(c);
                mode = Mode::NotEscaped;
            }
        }
    }
    string
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketOrQuote{
    None,
    OpenBracket(u32),
    SingleQuote,
    DoubleQuote
}


pub fn first_author_field_content(mut field: &str) -> &str
{
    let mut bracket_or_quote = BracketOrQuote::None;
    let mut start = 0;
    let mut iter = field.char_indices();
    
    while let Some((index, char)) = iter.next(){

        if char == '\\' {
            // skip next char
            let _ = iter.next();
            continue;
        }

        let mut set_field = || {
            field = match iter.next(){
                Some((idx, _)) => {
                    &field[start..idx]
                },
                None => {
                    &field[start..]
                }
            };
        };

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
                    set_field();
                    break;
                }
            },
            BracketOrQuote::SingleQuote => {
                if char == '\'' {
                    set_field();
                    break;
                }
            },
            BracketOrQuote::DoubleQuote => {
                if char == '"' {
                    set_field();
                    break;
                }
            }
        }
    }

    field
}

pub fn first_author_first_name(content: &str) -> String
{
    let first_author_field_content = first_author_from_content(content);
    let first_author_field_content = first_author_field_content.trim(); // also trim potential spaces
    // author field might contain one or more "," - like author = {Feld, Yannick AND Hartmann, Alexander K.}
    // We already removed all other authors, so this should contain at most one ","

    let mut iter = first_author_field_content.split(',');
    let first_entry = iter.next().unwrap();
    let second_entry = iter.next();
    assert_eq!(
        iter.next(), 
        None,
        "To many ',' in first author of a bib entry: {content}\nauthor_field: {}",
        first_author_field_content
    );
    match second_entry{
        None => {
            first_entry.trim().to_owned()
        },
        Some(first_name) => {
            format!("{} {}", first_name.trim(), first_entry.trim())
        }
    }

}

pub fn sort_by_first_author_field<F>(to_sort: &mut [BibEntry], case_fn: F)
where F: Fn(String) -> String
{
    to_sort
        .sort_by_cached_key(|entry| case_fn(first_author_from_content(&entry.content)));
}

pub fn sort_by_first_author_first_name<F>(to_sort: &mut [BibEntry], case_fn: F)
where F: Fn(String) -> String
{
    to_sort
        .sort_by_cached_key(|entry| case_fn(first_author_first_name(&entry.content)));
}