use lazy_static::lazy_static;
use regex::Regex;
use time::{macros::format_description, PrimitiveDateTime};

use crate::commit::Commit;

#[derive(Debug, PartialEq, Eq)]
pub enum ParsingError {
    ExpectedMoreInput,
    NoMatch(String),
    BadMatch,
    BadDate,
}

enum State {
    Start,
    Hash,
    Author,
    Date,
    Stats,
    Accept,
}

lazy_static! {
    // TODO Fix so hash doesn't include `(HEAD -> main)`. Will require
    // additional and/or updated tests.
    static ref HASH_REGEX: Regex = Regex::new(r"^commit (.+)$").unwrap();
    static ref AUTHOR_REGEX: Regex = Regex::new(r"^Author: (.+) <(.+)>$").unwrap();
    static ref DATE_REGEX: Regex = Regex::new(r"^Date:(.+)$").unwrap();
    static ref FILES_REGEX: Regex = Regex::new(r"(\d+) files? changed.+$").unwrap();
    static ref INSERTS_REGEX: Regex = Regex::new(r"\s(\d+) insertions?.+$").unwrap();
    static ref DELETES_REGEX: Regex = Regex::new(r"\s(\d+) deletions?.+$").unwrap();
}

pub fn parse(input: &str) -> Result<Vec<Commit>, ParsingError> {
    let mut result = Vec::new();

    let mut state = State::Hash;
    let mut commit = Commit::default();

    let mut lines = input.split("\n").peekable();

    loop {
        match state {
            State::Start => match lines.peek() {
                // When there are no more lines in the start state then the
                // machine terminates successfully
                None => break,
                // Ignore whitespace lines by consuming the line and continuing
                // to the next
                Some(line) => {
                    if line.trim().len() == 0 {
                        let _blank = lines.next();
                        continue;
                    }

                    state = State::Hash
                }
            },
            State::Hash => {
                let line = lines.next().ok_or(ParsingError::ExpectedMoreInput)?;

                let hash = one_match(&HASH_REGEX, line)?;

                commit.hash = hash;
                state = State::Author;
            }
            State::Author => {
                let line = lines.next().ok_or(ParsingError::ExpectedMoreInput)?;

                let (name, email) = two_matches(&AUTHOR_REGEX, line)?;

                commit.name = name;
                commit.email = email;

                state = State::Date;
            }
            State::Date => {
                let line = lines.next().ok_or(ParsingError::ExpectedMoreInput)?;

                let date = one_match(&DATE_REGEX, line)?;
                let date = date.trim();

                let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
                let date =
                    PrimitiveDateTime::parse(date, format).map_err(|_| ParsingError::BadDate)?;

                commit.date = date;
                state = State::Stats;
            }
            State::Stats => {
                let line = lines.next().ok_or(ParsingError::ExpectedMoreInput)?;

                let files = one_match(&FILES_REGEX, line);
                let inserts = one_match(&INSERTS_REGEX, line);
                let deletes = one_match(&DELETES_REGEX, line);

                if files.is_err() && inserts.is_err() && deletes.is_err() {
                    continue;
                }

                let files = files.unwrap_or_default();
                let files = files.parse::<u32>().unwrap_or_default();

                let inserts = inserts.unwrap_or_default();
                let inserts = inserts.parse::<u32>().unwrap_or_default();

                let deletes = deletes.unwrap_or_default();
                let deletes = deletes.parse::<u32>().unwrap_or_default();

                commit.files = files;
                commit.inserts = inserts;
                commit.deletes = deletes;

                state = State::Accept;
            }
            State::Accept => {
                result.push(commit.clone());
                state = State::Start;
            }
        }
    }

    Ok(result)
}

fn one_match(regex: &Regex, line: &str) -> Result<String, ParsingError> {
    match regex.captures(line) {
        None => {
            let message = format!("Match {:?} on {:?} yielded None", regex, line);
            Err(ParsingError::NoMatch(message))
        }
        Some(cap) => {
            let mat = cap.get(1).ok_or(ParsingError::BadMatch)?;
            let val = mat.as_str().to_string();

            Ok(val)
        }
    }
}

fn two_matches(regex: &Regex, line: &str) -> Result<(String, String), ParsingError> {
    match regex.captures(line) {
        None => {
            let message = format!("Match {:?} on {:?} yielded None", regex, line);
            Err(ParsingError::NoMatch(message))
        }
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
    #[test]
    fn all_stats() {
        let input = r"commit a75c00d4baa851fbd03d514cd980c999153fc21f
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

    #[test]
    fn no_deletes() {
        let input = r"commit a75
Author: Jonathan Neufeld <jneufeld@alumni.ubc.ca>
Date:   2022-11-24 22:11:50

  Refactor parser error handling

  1 file changed, 43 insertions(+)";

        match super::parse(input) {
            Err(why) => panic!("Error parsing commit because {:?}", why),
            Ok(commits) => {
                assert_eq!(commits.len(), 1);

                let commit = commits.get(0).unwrap();

                assert_eq!(commit.hash, "a75");
                assert_eq!(commit.name, "Jonathan Neufeld");
                assert_eq!(commit.email, "jneufeld@alumni.ubc.ca");
                assert_eq!(commit.files, 1);
                assert_eq!(commit.inserts, 43);
                assert_eq!(commit.deletes, 0);
            }
        }
    }

    #[test]
    fn no_inserts() {
        let input = r"commit a75
Author: Jonathan Neufeld <jneufeld@alumni.ubc.ca>
Date:   2022-11-24 22:11:50

  Refactor parser error handling

  1 file changed, 62 deletions(-)";

        match super::parse(input) {
            Err(why) => panic!("Error parsing commit because {:?}", why),
            Ok(commits) => {
                assert_eq!(commits.len(), 1);

                let commit = commits.get(0).unwrap();

                assert_eq!(commit.hash, "a75");
                assert_eq!(commit.name, "Jonathan Neufeld");
                assert_eq!(commit.email, "jneufeld@alumni.ubc.ca");
                assert_eq!(commit.files, 1);
                assert_eq!(commit.inserts, 0);
                assert_eq!(commit.deletes, 62);
            }
        }
    }

    #[test]
    fn two_commits() {
        let input = r"commit abc123
Author: Jon <jon@email.ca>
Date:   2022-11-24 22:11:50

  Do things

  1 file changed, 2 deletions(-)
  
commit def456
Author: Not Jon <notjon@email.org>
Date:   2022-11-24 22:11:50

  More things

  11 file changed, 22 insertions(+), 33 deletions(-)";

        match super::parse(input) {
            Err(why) => panic!("Error parsing commit because {:?}", why),
            Ok(commits) => {
                assert_eq!(commits.len(), 2);

                let commit = commits.get(0).unwrap();

                assert_eq!(commit.hash, "abc123");
                assert_eq!(commit.name, "Jon");
                assert_eq!(commit.email, "jon@email.ca");
                assert_eq!(commit.files, 1);
                assert_eq!(commit.inserts, 0);
                assert_eq!(commit.deletes, 2);

                let commit = commits.get(1).unwrap();

                assert_eq!(commit.hash, "def456");
                assert_eq!(commit.name, "Not Jon");
                assert_eq!(commit.email, "notjon@email.org");
                assert_eq!(commit.files, 11);
                assert_eq!(commit.inserts, 22);
                assert_eq!(commit.deletes, 33);
            }
        }
    }
}
