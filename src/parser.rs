use std::marker::PhantomData;

use lazy_static::lazy_static;
use regex::Regex;
use time::{macros::format_description, PrimitiveDateTime};

use crate::commit::Commit;

pub fn parse(input: &str) -> Result<Vec<Commit>, ParsingError> {
    let parser = Parser::default();

    for line in input.split("\n") {
        parser.parse(line);
    }

    todo!()
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParsingError {
    NoMatch,
    BadMatch,
    BadDate,
}

struct Hash;
struct Author;
struct Date;
struct Files;
struct Inserts;
struct Deletes;
struct Accept;

struct Parser<State> {
    state: PhantomData<State>,
    current: Commit,
    commits: Vec<Commit>,
}

impl Default for Parser<Hash> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            current: Default::default(),
            commits: Vec::new(),
        }
    }
}

lazy_static! {
    static ref HASH_REGEX: Regex = Regex::new(r"^commit (.+)$").unwrap();
}

impl Parser<Hash> {
    fn parse(self, line: &str) -> Result<Parser<Author>, ParsingError> {
        let hash = one_match(&HASH_REGEX, line)?;

        Ok(Parser::<Author> {
            state: PhantomData::<Author>,
            current: self.current.hash(&hash),
            commits: self.commits,
        })
    }
}

lazy_static! {
    static ref AUTHOR_REGEX: Regex = Regex::new(r"^Author: (.+) <(.+)>$").unwrap();
}

impl Parser<Author> {
    fn parse(self, line: &str) -> Result<Parser<Date>, ParsingError> {
        let (name, email) = two_matches(&AUTHOR_REGEX, line)?;

        Ok(Parser::<Date> {
            state: PhantomData::<Date>,
            current: self.current.name(&name).email(&email),
            commits: self.commits,
        })
    }
}

lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"^Date:(.+)$").unwrap();
}

impl Parser<Date> {
    fn parse(self, line: &str) -> Result<Parser<Files>, ParsingError> {
        let date = one_match(&DATE_REGEX, line)?;
        let date = date.trim();

        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let date = PrimitiveDateTime::parse(date, format).map_err(|_| ParsingError::BadDate)?;

        Ok(Parser::<Files> {
            state: PhantomData::<Files>,
            current: self.current.date(date),
            commits: self.commits,
        })
    }
}

lazy_static! {
    static ref FILES_REGEX: Regex = Regex::new(r"(\d+) files? changed.+$").unwrap();
}

impl Parser<Files> {
    fn parse(self, line: &str) -> Result<Parser<Inserts>, ParsingError> {
        let files = one_match(&FILES_REGEX, line)?;
        let files = files.parse::<u32>().unwrap();

        Ok(Parser::<Inserts> {
            state: PhantomData::<Inserts>,
            current: self.current.files(files),
            commits: self.commits,
        })
    }
}

lazy_static! {
    static ref INSERTS_REGEX: Regex = Regex::new(r"\s(\d+) insertions?.+$").unwrap();
}

impl Parser<Inserts> {
    fn parse(self, line: &str) -> Result<Parser<Deletes>, ParsingError> {
        let inserts = one_match(&INSERTS_REGEX, line)?;
        let inserts = inserts.parse::<u32>().unwrap();

        Ok(Parser::<Deletes> {
            state: PhantomData::<Deletes>,
            current: self.current.inserts(inserts),
            commits: self.commits,
        })
    }
}

lazy_static! {
    static ref DELETES_REGEX: Regex = Regex::new(r"\s(\d+) deletions?.+$").unwrap();
}

impl Parser<Deletes> {
    fn parse(self, line: &str) -> Result<Parser<Accept>, ParsingError> {
        let deletes = one_match(&DELETES_REGEX, line)?;
        let deletes = deletes.parse::<u32>().unwrap();

        Ok(Parser::<Accept> {
            state: PhantomData::<Accept>,
            current: self.current.deletes(deletes),
            commits: self.commits,
        })
    }
}

impl Parser<Accept> {
    fn parse(self) -> Result<Parser<Hash>, ParsingError> {
        Ok(Parser::<Hash> {
            state: PhantomData::<Hash>,
            current: Commit::default(),
            commits: self.commits,
        })
    }
}

fn one_match(regex: &Regex, line: &str) -> Result<String, ParsingError> {
    match regex.captures(line) {
        None => Err(ParsingError::NoMatch),
        Some(cap) => {
            let mat = cap.get(1).ok_or(ParsingError::BadMatch)?;
            let val = mat.as_str().to_string();

            Ok(val)
        }
    }
}

fn two_matches(regex: &Regex, line: &str) -> Result<(String, String), ParsingError> {
    match regex.captures(line) {
        None => Err(ParsingError::NoMatch),
        Some(cap) => {
            let first_mat = cap.get(1).ok_or(ParsingError::BadMatch)?;
            let first_val = String::from(first_mat.as_str());

            let second_mat = cap.get(2).ok_or(ParsingError::BadMatch)?;
            let second_val = String::from(second_mat.as_str());

            Ok((first_val, second_val))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use time::Month;

    use super::{Author, Date, Deletes, Files, Inserts, Parser, ParsingError};

    #[test]
    fn hash() {
        let parser = Parser::default();
        let line = "commit 9f617";

        match parser.parse(line) {
            Err(why) => panic!("Failed to parse hash ({}) because {:?}", line, why),
            Ok(result) => assert_eq!(result.current.hash, "9f617"),
        }
    }

    #[test]
    fn missing_hash() {
        let parser = Parser::default();
        let line = "dummy";

        match parser.parse(line) {
            Ok(_) => panic!("fail"),
            Err(why) => assert_eq!(why, ParsingError::NoMatch),
        }
    }

    #[test]
    fn author() {
        let parser = Parser::<Author> {
            state: PhantomData::<Author>,
            current: Default::default(),
            commits: Vec::new(),
        };

        let line = "Author: First Middle Last <email@alumni.ubc.ca>";

        match parser.parse(line) {
            Err(why) => panic!("Failed to parse hash ({}) because {:?}", line, why),
            Ok(result) => {
                assert_eq!(result.current.name, "First Middle Last");
                assert_eq!(result.current.email, "email@alumni.ubc.ca");
            }
        }
    }

    #[test]
    fn missing_author_name() {
        let parser = Parser::<Author> {
            state: PhantomData::<Author>,
            current: Default::default(),
            commits: Vec::new(),
        };

        let line = "Author: <email@alumni.ubc.ca>";

        match parser.parse(line) {
            Ok(_) => panic!("fail"),
            Err(why) => assert_eq!(why, ParsingError::NoMatch),
        }
    }

    #[test]
    fn missing_author_email() {
        let parser = Parser::<Author> {
            state: PhantomData::<Author>,
            current: Default::default(),
            commits: Vec::new(),
        };

        let line = "Author: First Middle Last";

        match parser.parse(line) {
            Ok(_) => panic!("fail"),
            Err(why) => assert_eq!(why, ParsingError::NoMatch),
        }
    }

    #[test]
    fn date() {
        let parser = Parser::<Date> {
            state: PhantomData::<Date>,
            current: Default::default(),
            commits: Vec::new(),
        };

        let line = "Date:   2022-11-24 21:07:24";

        match parser.parse(line) {
            Err(why) => panic!("Failed to parse date ({}) because {:?}", line, why),
            Ok(result) => {
                let date = result.current.date;

                assert_eq!(date.year(), 2022);
                assert_eq!(date.month(), Month::November);
                assert_eq!(date.day(), 24);
            }
        }
    }

    #[test]
    fn missing_date() {
        let parser = Parser::<Date> {
            state: PhantomData::<Date>,
            current: Default::default(),
            commits: Vec::new(),
        };

        let line = "Thursday";

        match parser.parse(line) {
            Ok(_) => panic!("fail"),
            Err(why) => assert_eq!(why, ParsingError::NoMatch),
        }
    }

    #[test]
    fn files() {
        let parser = Parser::<Files> {
            current: Default::default(),
            state: PhantomData::<Files>,
            commits: Vec::new(),
        };

        let line = "1 files changed, 2 insertions(+), 3 deletions(-)";

        match parser.parse(line) {
            Err(why) => panic!("Failed to parse hash ({}) because {:?}", line, why),
            Ok(result) => assert_eq!(result.current.files, 1),
        }
    }

    #[test]
    fn inserts() {
        let parser = Parser::<Inserts> {
            state: PhantomData::<Inserts>,
            current: Default::default(),
            commits: Vec::new(),
        };

        let line = "1 files changed, 2 insertions(+), 3 deletions(-)";

        match parser.parse(line) {
            Err(why) => panic!("Failed to parse hash ({}) because {:?}", line, why),
            Ok(result) => assert_eq!(result.current.inserts, 2),
        }
    }

    #[test]
    fn deletes() {
        let parser = Parser::<Deletes> {
            state: PhantomData::<Deletes>,
            current: Default::default(),
            commits: Vec::new(),
        };

        let line = "1 files changed, 2 insertions(+), 3 deletions(-)";

        match parser.parse(line) {
            Err(why) => panic!("Failed to parse hash ({}) because {:?}", line, why),
            Ok(result) => assert_eq!(result.current.deletes, 3),
        }
    }

    #[test]
    fn message_line() {
        let parser = Parser::<Files> {
            current: Default::default(),
            state: PhantomData::<Files>,
            commits: Vec::new(),
        };

        let line = "Refactor parser error handling";

        match parser.parse(line) {
            Ok(_) => panic!("fail"),
            Err(why) => assert_eq!(why, ParsingError::NoMatch),
        }
    }

    #[test]
    fn single_commit() {
        let input = r"commit a75c00d4baa851fbd03d514cd980c999153fc21f (HEAD -> main)
Author: Jonathan Neufeld <jneufeld@alumni.ubc.ca>
Date:   2022-11-24 22:11:50

  Refactor parser error handling

  src/parser.rs | 105 +++++++++++++++++++++++++++++++++++++++++++--------------------------------------------------------------
  1 file changed, 43 insertions(+), 62 deletions(-)";

        match super::parse(input) {
            Err(why) => panic!("Error parsing commit because {:?}", why),
            Ok(commits) => {
                assert_eq!(commits.len(), 1);

                let commit = commits.get(0).unwrap();

                assert_eq!(commit.hash, "a75c00d4baa851fbd03d514cd980c999153fc21f");
                assert_eq!(commit.name, "Jonathan Neufeld");
                assert_eq!(commit.email, "jneufeld@alumni.ubc.ca");
                assert_eq!(commit.files, 1);
                assert_eq!(commit.inserts, 43);
                assert_eq!(commit.deletes, 62);
            }
        }
    }
}
