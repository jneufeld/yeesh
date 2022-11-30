mod commit;
mod histogram;
mod parser;

use std::process::{self, Command};
use std::str;

use crate::commit::Commit;
use crate::histogram::Kind;

use termion::{color, style};

const HELP: &str = "\
yeesh: simple stats for git repositories

USAGE:
  yeesh [-h] [--hours] [--days]

ARGS:
  -h, --help    Prints this message
  --hours       (Optional) prints commit stats by hour of day
  --days        (Optional) prints commit stats by weekday
";

#[derive(Debug)]
struct CliArgs {
    hours: bool,
    days: bool,
}

fn main() {
    let args = args_or_quit();

    let logs = get_git_logs();
    let commits = parser::parse(&logs);
    let commits = commits.unwrap();

    if args.hours {
        print_hours(&commits);
    }

    if args.days {
        print_weekdays(&commits);
    }
}

fn args_or_quit() -> CliArgs {
    let args = parse_cli_args();

    if !args.days && !args.hours {
        print_help_and_quit();
    }

    args
}

fn parse_cli_args() -> CliArgs {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print_help_and_quit();
    }

    CliArgs {
        hours: args.contains("--hours"),
        days: args.contains("--days"),
    }
}

fn print_help_and_quit() {
    print!("{}", HELP);
    process::exit(1);
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

fn print_hours(commits: &Vec<Commit>) {
    println!(
        "{}{}By hour:{}",
        style::Bold,
        color::Fg(color::Magenta),
        style::Reset
    );

    let by_hour = histogram::of_kind(Kind::ByHour, commits);

    for hour in 1..24 {
        let count = by_hour.count_at(hour) as usize;

        println!(
            "{}{:02} {}| {}{}{}",
            color::Fg(color::LightBlue),
            hour,
            color::Fg(color::White),
            color::Fg(color::Yellow),
            "-".repeat(count),
            style::Reset,
        );
    }

    println!(
        "\n{}total: {}{}\n",
        style::Faint,
        by_hour.len(),
        style::Reset
    );
}

fn print_weekdays(commits: &Vec<Commit>) {
    println!(
        "{}{}By weekday:{}",
        style::Bold,
        color::Fg(color::Magenta),
        style::Reset
    );

    let by_weekday = histogram::of_kind(Kind::ByWeekday, commits);

    for weekday in 1..7 {
        let count = by_weekday.count_at(weekday) as usize;

        println!(
            "{}{:02} {}| {}{}{}",
            color::Fg(color::LightBlue),
            weekday,
            color::Fg(color::White),
            color::Fg(color::Yellow),
            "-".repeat(count),
            style::Reset,
        );
    }

    println!(
        "\n{}total: {}{}\n",
        style::Faint,
        by_weekday.len(),
        style::Reset
    );
}
