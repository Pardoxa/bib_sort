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

            author_field_next = field_content(author_field_next);

            if let Some(and_match) =  AND.find(author_field_next) {
                author_field_next = &author_field_next[..and_match.start()];
            }

            clean_string(author_field_next)

        }
    }
}

pub fn clean_string(s: &str) -> String
{
    let mut string = String::with_capacity(s.len());
    let mut iter = s.chars();
    while let Some(c) = iter.next()
    {
        match c {
            '\\' => {
                let _ = iter.next();
                continue;
            },
            '{' | '}' | '\'' | '"' => continue,
            _ => {
                string.push(c);
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


pub fn field_content(mut field: &str) -> &str
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

pub fn sort_by_first_author_field<F>(to_sort: &mut [BibEntry], case_fn: F)
where F: Fn(String) -> String
{
    to_sort
        .sort_by_cached_key(|entry| case_fn(first_author_from_content(&entry.content)));
}