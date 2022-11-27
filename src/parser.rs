use anyhow::Context;
use lazy_static::lazy_static;
use regex::Regex;
use time::{macros::format_description, PrimitiveDateTime};

use crate::commit::{Author, Commit};

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
pub fn parse(input: &str) -> anyhow::Result<Vec<Commit>> {
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

fn parse_hash(line: Option<&str>) -> anyhow::Result<String> {
    let message = format!(
        "Expected line to parse commit hash from on input {:?} but got None",
        line
    );

    let line = line.context(message)?;
    let hash = one_match(&HASH_REGEX, line)?;

    Ok(hash)
}

fn parse_author(line: Option<&str>) -> anyhow::Result<Author> {
    let message = format!(
        "Expected line to parse author from on input {:?} but got None",
        line
    );

    let line = line.context(message)?;

    // TODO use the `newtype` idiom so destructuring the tuple is harder to get
    // wrong, i.e. confuse the order of two `String`s
    let (name, email) = two_matches(&AUTHOR_REGEX, line)?;

    let author = Author::new(name, email);

    Ok(author)
}

fn parse_date(line: Option<&str>) -> anyhow::Result<PrimitiveDateTime> {
    let message = format!(
        "Expected line to parse date from on input {:?} but got None",
        line
    );

    let line = line.context(message)?;
    let date = one_match(&DATE_REGEX, line)?;

    // git's output likely contains whitespace that isn't relevant to this
    // program's parsing. So far, it hasn't been cleaned/modified in any way.
    // Call `trim()` to remove such whitespace.
    let date = date.trim();

    // Yes, it's sort of hard-coding, but `year`, `month`, etc. are in the API
    // documentation as an example
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

    let message = format!(
        "Expected line to parse date from on input {:?} but got None",
        line
    );

    let date = PrimitiveDateTime::parse(date, format).context(message)?;

    Ok(date)
}

// TODO is it more idiomatic to return `usize` when I can't see a need for the
// particular sizing? Otherwise, the restricting the return values to
// non-negative should be sufficient.
fn parse_stat(regex: &Regex, line: Option<&str>) -> anyhow::Result<u32> {
    let message = format!(
        "Expected line to parse stat {:?} on input {:?} but got None",
        regex, line
    );

    let line = line.context(message)?;

    let stat = one_match(regex, line)?;
    let stat = stat.parse::<u32>().unwrap_or_default();

    Ok(stat)
}

/// Expect a single match from the given regular expression on the input. Return
/// the value as a `String` or a parsing error.
fn one_match(regex: &Regex, line: &str) -> anyhow::Result<String> {
    let message = format!(
        "Expected first match from {:?} on input {:?} but got None",
        regex, line
    );

    let captures = regex.captures(line).context(message)?;

    // The 0th capture, i.e. `get(0)`, returns all captures. That may seem a little
    // unusual, but is specifically mentioned in the API documentation. This
    // comment passes that relavent information along.
    let first_match = captures
        .get(1)
        .context("Can't match on the first part of the regex")?;

    let value = String::from(first_match.as_str());

    Ok(value)
}

/// Expect two matches from the given regular expression on the input. Return
/// the value as a `String` or a parsing error.
fn two_matches(regex: &Regex, line: &str) -> anyhow::Result<(String, String)> {
    let message = format!(
        "Expected first two matches from {:?} on input {:?} but got None",
        regex, line
    );

    let captures = regex.captures(line).context(message)?;

    // The 0th capture, i.e. `get(0)`, returns all captures. That may seem a little
    // unusual, but is specifically mentioned in the API documentation. This
    // comment passes that relavent information along.
    //
    // TODO When worthwhile documentation about APIs is copy/pasted it suggests
    // there may be a need to refactor. See `one_match()` above.
    let first_match = captures
        .get(1)
        .context("Can't match on the first part of the regex")?;
    let second_match = captures
        .get(2)
        .context("Can't match on the second part of the regex")?;

    let first_value = String::from(first_match.as_str());
    let second_val = String::from(second_match.as_str());

    Ok((first_value, second_val))
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
