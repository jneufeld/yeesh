use lazy_static::lazy_static;
use regex::Regex;
use time::{macros::format_description, PrimitiveDateTime};

use crate::commit::{Author, Commit};

/// Meager attempt to indicate what went wrong while parsing git logs
#[derive(Debug, PartialEq, Eq)]
pub enum ParsingError {
    /// Used when parsing started for a commit but not all required fields have
    /// been parsed. E.g. the input contains only the hash and author but no
    /// message or stats.
    ExpectedMoreInput,

    /// Indicates an expected field isn't present. E.g. the author's name and
    /// email follow the hash but aren't present.
    NoMatch(String),

    /// Parsing found the right type of line but wasn't able to extract the
    /// required information. E.g. the commit's stats are present but not in the
    /// expected format.
    BadMatch,

    /// Indicates the date is present but in an unexpected format
    BadDate,
}

/// Represents the state machine's current state
enum State {
    /// Used to successfully terminate if no more lines are available for
    /// parsing or prepare the state machine to parse a commit
    Start,

    /// Indicates the parser expects the next line to contain the commit's hash
    Hash,

    /// Indicates the parser expects the next line to contain the author's name
    /// and email
    Author,

    /// Indicates the parser expects the next line to contain the date
    Date,

    /// Indicates the parser expects the next line to contain the number of
    /// files modified, insertions, and deletions
    Stats,

    /// Used to clean up success parsing of a commit
    Accept,
}

// Compile regular expressions only once and at compile time
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

/// Given an input in the format of git logs this will return the commits. If
/// parsing fails the result contains a meaningful error.
///
/// Note that the input format is specific. That is, the git logs must contain
/// stats via `--stat` and a particular date format. These are defined in
/// `main.rs` and are tightly coupled to the implementation. In other words,
/// this is brittle!
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
                commit.hash = parse_hash(lines.next())?;
                state = State::Author;
            }
            State::Author => {
                commit.author = parse_author(lines.next())?;
                state = State::Date;
            }
            State::Date => {
                commit.date = parse_date(lines.next())?;
                state = State::Stats;
            }
            State::Stats => {
                let line = lines.next();

                let files = parse_stat(&FILES_REGEX, line);
                let inserts = parse_stat(&INSERTS_REGEX, line);
                let deletes = parse_stat(&DELETES_REGEX, line);

                if files.is_err() && inserts.is_err() && deletes.is_err() {
                    continue;
                }

                commit.files = files.unwrap_or_default();
                commit.inserts = inserts.unwrap_or_default();
                commit.deletes = deletes.unwrap_or_default();

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

fn parse_hash(line: Option<&str>) -> Result<String, ParsingError> {
    let line = line.ok_or(ParsingError::ExpectedMoreInput)?;
    let hash = one_match(&HASH_REGEX, line)?;

    Ok(hash)
}

fn parse_author(line: Option<&str>) -> Result<Author, ParsingError> {
    let line = line.ok_or(ParsingError::ExpectedMoreInput)?;
    let (name, email) = two_matches(&AUTHOR_REGEX, line)?;
    let author = Author::new(name, email);

    Ok(author)
}

fn parse_date(line: Option<&str>) -> Result<PrimitiveDateTime, ParsingError> {
    let line = line.ok_or(ParsingError::ExpectedMoreInput)?;

    let date = one_match(&DATE_REGEX, line)?;
    let date = date.trim();

    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let date = PrimitiveDateTime::parse(date, format).map_err(|_| ParsingError::BadDate)?;

    Ok(date)
}

fn parse_stat(regex: &Regex, line: Option<&str>) -> Result<u32, ParsingError> {
    let line = line.ok_or(ParsingError::ExpectedMoreInput)?;

    let stat = one_match(regex, line)?;
    let stat = stat.parse::<u32>().unwrap_or_default();

    Ok(stat)
}

/// Expect a single match from the given regular expression on the input. Return
/// the value as a `String` or a parsing error.
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

/// Expect two matches from the given regular expression on the input. Return
/// the value as a `String` or a parsing error.
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
                assert_eq!(commit.author.name, "Jonathan Neufeld");
                assert_eq!(commit.author.email, "jneufeld@alumni.ubc.ca");
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
                assert_eq!(commit.author.name, "Jonathan Neufeld");
                assert_eq!(commit.author.email, "jneufeld@alumni.ubc.ca");
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
                assert_eq!(commit.author.name, "Jonathan Neufeld");
                assert_eq!(commit.author.email, "jneufeld@alumni.ubc.ca");
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
                assert_eq!(commit.author.name, "Jon");
                assert_eq!(commit.author.email, "jon@email.ca");
                assert_eq!(commit.files, 1);
                assert_eq!(commit.inserts, 0);
                assert_eq!(commit.deletes, 2);

                let commit = commits.get(1).unwrap();

                assert_eq!(commit.hash, "def456");
                assert_eq!(commit.author.name, "Not Jon");
                assert_eq!(commit.author.email, "notjon@email.org");
                assert_eq!(commit.files, 11);
                assert_eq!(commit.inserts, 22);
                assert_eq!(commit.deletes, 33);
            }
        }
    }
}
