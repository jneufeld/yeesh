use hdrhistogram::Histogram;

use crate::commit::Commit;

pub enum Kind {
    ByHour,
    ByWeekday,
}

pub fn of_kind(kind: Kind, commits: &Vec<Commit>) -> Histogram<u8> {
    match kind {
        Kind::ByHour => by_hour(commits),
        Kind::ByWeekday => by_weekday(commits),
    }
}

fn by_hour(commits: &Vec<Commit>) -> Histogram<u8> {
    let mut histogram = Histogram::new_with_bounds(1, 24, 1).unwrap();

    for commit in commits {
        let hour = get_hour(commit);
        histogram.record(hour).unwrap();
    }

    histogram
}

fn by_weekday(commits: &Vec<Commit>) -> Histogram<u8> {
    let mut histogram = Histogram::new_with_bounds(1, 7, 1).unwrap();

    for commit in commits {
        let weekday = get_weekday(commit);
        histogram.record(weekday).unwrap();
    }

    histogram
}

fn get_hour(commit: &Commit) -> u64 {
    commit.date.hour() as u64
}

fn get_weekday(commit: &Commit) -> u64 {
    match commit.date.weekday() {
        time::Weekday::Monday => 1,
        time::Weekday::Tuesday => 2,
        time::Weekday::Wednesday => 3,
        time::Weekday::Thursday => 4,
        time::Weekday::Friday => 5,
        time::Weekday::Saturday => 6,
        time::Weekday::Sunday => 7,
    }
}
