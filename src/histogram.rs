use crate::commit::Commit;

pub struct Histogram {
    histogram: hdrhistogram::Histogram<u8>,
}

pub enum Kind {
    ByHour,
    ByWeekday,
}

impl Histogram {
    pub fn new(kind: Kind, commits: &Vec<Commit>) -> Histogram {
        // TODO obviously refactor this... yeesh
        let histogram = match kind {
            Kind::ByHour => {
                let mut h = hdrhistogram::Histogram::new_with_bounds(1, 24, 1).unwrap();

                for commit in commits {
                    let hour = get_hour(&commit);
                    h.record(hour).unwrap();
                }

                h
            }
            Kind::ByWeekday => {
                let mut h = hdrhistogram::Histogram::new_with_bounds(1, 7, 1).unwrap();

                for commit in commits {
                    let weekday = get_weekday(&commit);
                    h.record(weekday).unwrap();
                }

                h
            }
        };

        Histogram { histogram }
    }

    pub fn min(&self) -> u64 {
        self.histogram.min_nz()
    }

    pub fn max(&self) -> u64 {
        self.histogram.max()
    }

    pub fn mean(&self) -> f64 {
        self.histogram.mean()
    }

    pub fn median(&self) -> u64 {
        self.histogram.value_at_percentile(50.0)
    }

    pub fn std_dev(&self) -> f64 {
        self.histogram.stdev()
    }

    pub fn len(&self) -> u64 {
        self.histogram.len()
    }
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
