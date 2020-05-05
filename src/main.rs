mod commit;
mod commit_builder;
mod log_parser;

use std::process::Command;
use std::str;

fn main() {
    let logs = get_git_logs();
    let commits = log_parser::to_commits(&logs);

    for commit in commits {
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

    git_logs.to_string()
}
