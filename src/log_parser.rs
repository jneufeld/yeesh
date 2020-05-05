use crate::commit::Commit;
use crate::commit_builder::CommitBuilder;

use regex::Regex;

pub fn to_commits(logs: &str) -> Vec<Commit> {
    let mut commits: Vec<Commit> = Vec::new();

    let re_hash = Regex::new(r"^commit (.+)$").unwrap();
    let re_auth = Regex::new(r"^Author: (.+) <(.+)>$").unwrap();
    let re_date = Regex::new(r"^Date:(.+)$").unwrap();
    let re_files = Regex::new(r"(\d+) files? changed.+$").unwrap();
    let re_inserts = Regex::new(r"\s(\d+) insertions?.+$").unwrap();
    let re_deletes = Regex::new(r"\s(\d+) deletions?.+$").unwrap();

    let mut commit_builder = CommitBuilder::new();

    for line in logs.split('\n') {
        let mut should_build = false;

        commit_builder = match get_one(&re_hash, &line) {
            None => commit_builder,
            Some(h) => commit_builder.hash(h),
        };

        commit_builder = match get_two(&re_auth, &line) {
            None => commit_builder,
            Some((name, email)) => commit_builder.name(name).email(email),
        };

        commit_builder = match get_one(&re_date, &line) {
            None => commit_builder,
            Some(d) => commit_builder.date(d.trim().to_string()),
        };

        commit_builder = match get_one(&re_files, &line) {
            None => commit_builder,
            Some(f) => {
                should_build = true;
                commit_builder.files(f.parse::<u32>().unwrap())
            }
        };

        commit_builder = match get_one(&re_inserts, &line) {
            None => commit_builder,
            Some(i) => commit_builder.inserts(i.parse::<u32>().unwrap()),
        };

        commit_builder = match get_one(&re_deletes, &line) {
            None => commit_builder,
            Some(d) => commit_builder.deletes(d.parse::<u32>().unwrap()),
        };

        if should_build {
            let commit = commit_builder.build();
            commits.push(commit);
            commit_builder = CommitBuilder::new();
        }
    }

    commits
}

fn get_one(re: &Regex, line: &str) -> Option<String> {
    let cap = re.captures(&line);

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
    let cap = re.captures(&line);

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
