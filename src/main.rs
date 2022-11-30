mod commit;
mod histogram;
mod parser;

use std::process::{self, Command};
use std::str;

use crate::commit::Commit;
use crate::histogram::Kind;

const HELP: &str = "\
yeesh: simple stats for git repositories

USAGE:
  yeesh [-h]

ARGS:
  -h, --help    Prints this message
";

fn main() {
    check_args();

    let logs = get_git_logs();

    match parser::parse(&logs) {
        Ok(commits) => print_histogram(commits),
        Err(why) => eprintln!("Error parsing git logs: {:?}", why),
    }
}

fn check_args() {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print!("{}", HELP);
        process::exit(1);
    }
}

fn get_git_logs() -> String {
    // The date format below yields the committer's local date. Regardless when
    // (or where) this program is run, the local time of the commit is what gets
    // captured. This is more meaningful than coverting dates and times into the
    // local timezone of the person running the tool.
    //
    // The following StackOverflow discussion has more details:
    // https://stackoverflow.com/questions/7853332/how-to-change-git-log-date-formats
    let proc_output = Command::new("git")
        .arg("log")
        .arg("--stat")
        .arg("--date=rfc")
        .output()
        .unwrap();

    let git_logs = str::from_utf8(&proc_output.stdout).unwrap();

    git_logs.to_string()
}

fn print_histogram(commits: Vec<Commit>) {
    println!("By hour:");

    let by_hour = histogram::of_kind(Kind::ByHour, &commits);

    for hour in 1..24 {
        let count = by_hour.count_at(hour) as usize;
        println!("{:02} | {}", hour, "-".repeat(count));
    }

    println!();

    println!("By weekday:");

    let by_weekday = histogram::of_kind(Kind::ByWeekday, &commits);

    for weekday in 1..7 {
        let count = by_weekday.count_at(weekday) as usize;
        println!("{:02} | {}", weekday, "-".repeat(count));
    }

    println!("total: {}", by_hour.len());
}
