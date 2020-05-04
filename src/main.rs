use std::process::Command;
use std::str;

fn main() {
    let proc_output = Command::new("git")
        .arg("log")
        .arg("--stat")
        .output()
        .unwrap();

    let git_log = str::from_utf8(&proc_output.stdout).unwrap();

    println!("{}", &git_log);
}
