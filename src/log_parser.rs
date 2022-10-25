use crate::commit::Commit;

use chrono::DateTime;
use regex::Regex;

pub fn to_commits(logs: &str) -> Vec<Commit> {
    let mut commits: Vec<Commit> = Vec::new();

    let re_hash = Regex::new(r"^commit (.+)$").unwrap();
    let re_auth = Regex::new(r"^Author: (.+) <(.+)>$").unwrap();
    let re_date = Regex::new(r"^Date:(.+)$").unwrap();
    let re_files = Regex::new(r"(\d+) files? changed.+$").unwrap();
    let re_inserts = Regex::new(r"\s(\d+) insertions?.+$").unwrap();
    let re_deletes = Regex::new(r"\s(\d+) deletions?.+$").unwrap();

    let mut commit = Commit::new();

    for line in logs.split('\n') {
        let mut is_commit_ready = false;

        match get_one(&re_hash, line) {
            None => (),
            Some(h) => commit.hash = h,
        };

        match get_two(&re_auth, line) {
            None => (),
            Some((name, email)) => {
                commit.name = name;
                commit.email = email;
            }
        };

        match get_one(&re_date, line) {
            None => (),
            Some(d) => {
                let date_time = DateTime::parse_from_rfc2822(&d);
                let date_time = date_time.unwrap_or_else(|_| panic!("Unable to parse date: {}", d));
                commit.date = Some(date_time);
            }
        };

        match get_one(&re_files, line) {
            None => (),
            Some(f) => {
                is_commit_ready = true;
                commit.files = f.parse::<u32>().unwrap();
            }
        };

        match get_one(&re_inserts, line) {
            None => (),
            Some(i) => commit.inserts = i.parse::<u32>().unwrap(),
        };

        match get_one(&re_deletes, line) {
            None => (),
            Some(d) => commit.deletes = d.parse::<u32>().unwrap(),
        };

        if is_commit_ready {
            commits.push(commit);
            commit = Commit::new();
        }
    }

    commits
}

fn get_one(re: &Regex, line: &str) -> Option<String> {
    let cap = re.captures(line);

    match cap {
        None => None,
        Some(cap) => {
            let mat = cap.get(1)?;
            let val = mat.as_str().to_string();

            Some(val)
        }
    }
}

fn get_two(re: &Regex, line: &str) -> Option<(String, String)> {
    let cap = re.captures(line);

    match cap {
        None => None,
        Some(cap) => {
            let first_mat = cap.get(1)?;
            let first_val = first_mat.as_str().to_string();

            let second_mat = cap.get(2)?;
            let second_val = second_mat.as_str().to_string();

            Some((first_val, second_val))
        }
    }
}
