use std::marker::PhantomData;

use regex::Regex;

#[derive(Debug, PartialEq)]
enum ParsingError {
    NoMatch,
    BadMatch,
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
}

impl Default for Parser<Hash> {
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

impl Parser<Hash> {
    pub fn parse(&self, line: &str) -> Result<Parser<Author>, ParsingError> {
        let regex = Regex::new(r"^commit (.+)$").unwrap();

        match one_match(&regex, line) {
            Err(why) => Err(why),
            Ok(_value) => Ok(Parser::<Author> {
                state: PhantomData::<Author>,
            }),
        }
    }
}

impl Parser<Author> {
    pub fn parse(&self, line: &str) -> Result<Parser<Date>, ParsingError> {
        let regex = Regex::new(r"^Author: (.+) <(.+)>$").unwrap();

        match two_matches(&regex, line) {
            Err(why) => Err(why),
            Ok((_first, _second)) => Ok(Parser::<Date> {
                state: PhantomData::<Date>,
            }),
        }
    }
}

impl Parser<Date> {
    pub fn parse(&self, line: &str) -> Result<Parser<Files>, ParsingError> {
        let regex = Regex::new(r"^Date:(.+)$").unwrap();

        match one_match(&regex, line) {
            Err(why) => Err(why),
            Ok(_value) => Ok(Parser::<Files> {
                state: PhantomData::<Files>,
            }),
        }
    }
}

impl Parser<Files> {
    pub fn parse(&self, line: &str) -> Result<Parser<Inserts>, ParsingError> {
        let regex = Regex::new(r"(\d+) files? changed.+$").unwrap();

        match one_match(&regex, line) {
            Err(why) => Err(why),
            Ok(_value) => Ok(Parser::<Inserts> {
                state: PhantomData::<Inserts>,
            }),
        }
    }
}

impl Parser<Inserts> {
    pub fn parse(&self, line: &str) -> Result<Parser<Deletes>, ParsingError> {
        let regex = Regex::new(r"\s(\d+) insertions?.+$").unwrap();

        match one_match(&regex, line) {
            Err(why) => Err(why),
            Ok(_value) => Ok(Parser::<Deletes> {
                state: PhantomData::<Deletes>,
            }),
        }
    }
}

impl Parser<Deletes> {
    pub fn parse(&self, line: &str) -> Result<Parser<Accept>, ParsingError> {
        let regex = Regex::new(r"\s(\d+) deletions?.+$").unwrap();

        match one_match(&regex, line) {
            Err(why) => Err(why),
            Ok(_value) => Ok(Parser::<Accept> {
                state: PhantomData::<Accept>,
            }),
        }
    }
}

impl Parser<Accept> {
    pub fn parse(&self) -> Parser<Hash> {
        todo!()
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

    use crate::parser::{Author, Deletes, Files, Inserts};

    use super::{Date, Parser, ParsingError};

    #[test]
    fn hash() {
        let parser = Parser::default();
        let line = "commit 9f617";

        let result = parser.parse(line);

        assert!(result.is_ok());
    }

    #[test]
    fn missing_hash() {
        let parser = Parser::default();
        let line = "9f617";

        match parser.parse(line) {
            Ok(_) => panic!("fail"),
            Err(why) => assert_eq!(why, ParsingError::NoMatch),
        }
    }

    #[test]
    fn author() {
        let parser = Parser::<Author> {
            state: PhantomData::<Author>,
        };
        let line = "Author: First Middle Last <email@alumni.ubc.ca>";

        let result = parser.parse(line);

        assert!(result.is_ok());
    }

    #[test]
    fn missing_author_name() {
        let parser = Parser::<Author> {
            state: PhantomData::<Author>,
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
        };
        let line = "Date: Thu Nov 24 14:14:44 2022 -0800";

        let result = parser.parse(line);

        assert!(result.is_ok());
    }

    #[test]
    fn missing_date() {
        let parser = Parser::<Date> {
            state: PhantomData::<Date>,
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
            state: PhantomData::<Files>,
        };
        let line = "3 files changed, 37 insertions(+), 3 deletions(-)";

        let result = parser.parse(line);

        assert!(result.is_ok());
    }

    #[test]
    fn inserts() {
        let parser = Parser::<Inserts> {
            state: PhantomData::<Inserts>,
        };
        let line = "3 files changed, 37 insertions(+), 3 deletions(-)";

        let result = parser.parse(line);

        assert!(result.is_ok());
    }

    #[test]
    fn deletes() {
        let parser = Parser::<Deletes> {
            state: PhantomData::<Deletes>,
        };
        let line = "3 files changed, 37 insertions(+), 3 deletions(-)";

        let result = parser.parse(line);

        assert!(result.is_ok());
    }
}
