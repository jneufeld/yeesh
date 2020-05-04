use std::process::Command;
use std::str;

use regex::Regex;

fn main() {
    let logs = get_git_logs();
    let commits = get_git_commits(&logs);

    for commit in commits.iter() {
        println!("{:#?}", &commit);
    }
}

fn get_git_logs() -> String {
    let proc_output = Command::new("git")
        .arg("log")
        .arg("--stat")
        .output()
        .unwrap();

    let git_logs = str::from_utf8(&proc_output.stdout).unwrap();
    let git_logs = git_logs.to_string();

    git_logs
}

#[derive(Debug)]
struct Commit {
    hash: String,
    name: String,
    email: String,
    date: String,
    files: u32,
    inserts: u32,
    deletes: u32
}

struct CommitBuilder {
    commit: Commit
}

impl CommitBuilder {
    fn new() -> CommitBuilder {
        let commit = Commit {
            hash: "".to_string(),
            name: "".to_string(),
            email: "".to_string(),
            date: "".to_string(),
            files: 0,
            inserts: 0,
            deletes: 0
        };

        CommitBuilder { commit }
    }

    fn hash(mut self, hash: String) -> CommitBuilder {
        self.commit.hash = hash;
        self
    }

    fn name(mut self, name: String) -> CommitBuilder {
        self.commit.name = name;
        self
    }

    fn email(mut self, email: String) -> CommitBuilder {
        self.commit.email = email;
        self
    }

    fn date(mut self, date: String) -> CommitBuilder {
        self.commit.date = date;
        self
    }

    fn files(mut self, files: u32) -> CommitBuilder {
        self.commit.files = files;
        self
    }

    fn inserts(mut self, inserts: u32) -> CommitBuilder {
        self.commit.inserts = inserts;
        self
    }

    fn deletes(mut self, deletes: u32) -> CommitBuilder {
        self.commit.deletes = deletes;
        self
    }

    fn build(self) -> Commit {
        self.commit
    }
}

fn get_git_commits(logs: &str) -> Vec<Commit> {
    let mut commits: Vec<Commit> = Vec::new();

    let re_hash = Regex::new(r"^commit (.+)$").unwrap();
    let re_auth = Regex::new(r"^Author: (.+) <(.+)>$").unwrap();
    let re_date = Regex::new(r"^Date:(.+)$").unwrap();
    let re_files = Regex::new(r"(\d+) files? changed.+$").unwrap();
    let re_inserts = Regex::new(r"\s(\d+) insertions.+$").unwrap();
    let re_deletes = Regex::new(r"\s(\d+) deletions.+$").unwrap();

    let mut commit_builder = CommitBuilder::new();

    for line in logs.split("\n") {
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
            },
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
        },
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
        },
    }
}
