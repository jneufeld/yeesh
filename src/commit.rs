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

impl Commit {
    pub fn new() -> Commit {
        Commit {
            hash: "".to_string(),
            name: "".to_string(),
            email: "".to_string(),
            date: None,
            files: 0,
            inserts: 0,
            deletes: 0,
        }
    }
}
