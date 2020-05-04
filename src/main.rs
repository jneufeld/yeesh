use std::process::Command;
use std::str;

fn main() {
    let git_log = get_git_logs();

    println!("{}", &git_log);
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
