mod commit;
mod log_parser;

use std::collections::HashMap;
use std::process::Command;
use std::str;

use chrono::Timelike;

use crate::commit::Commit;

fn main() {
    let logs = get_git_logs();
    let commits = log_parser::to_commits(&logs);

    print_by_hour(&commits);
}

fn get_git_logs() -> String {
    let proc_output = Command::new("git")
        .arg("log")
        .arg("--stat")
        .arg("--date=rfc2822")
        .output()
        .unwrap();

    let git_logs = str::from_utf8(&proc_output.stdout).unwrap();

    git_logs.to_string()
}

fn print_by_hour(commits: &Vec<Commit>) {
    // This is used twice in the function. I have already used wrong magic number
    // twice so have concluded it wise to define this here.
    let hours_in_day = 24;

    let mut by_hour = HashMap::new();

    for i in 0..hours_in_day {
        by_hour.insert(i, 0);
    }

    for commit in commits {
        let date = commit.date.unwrap();
        let hour = date.hour();

        // Getting the current value inside the `insert` is ugly but satisfies
        // a concerning compiler error message regarding mutable borrows.
        by_hour.insert(hour, 1 + by_hour.get(&hour).unwrap());
    }

    println!("Commits by hour:");

    for hour in 0..hours_in_day {
        let commits = by_hour.get(&hour).unwrap();
        println!("{:02}: {}", hour, commits);
    }
}
