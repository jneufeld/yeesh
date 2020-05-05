use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct Commit {
    pub hash: String,
    pub name: String,
    pub email: String,
    pub date: Option<DateTime<FixedOffset>>,
    pub files: u32,
    pub inserts: u32,
    pub deletes: u32,
}
