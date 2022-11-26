use time::{macros::datetime, PrimitiveDateTime};

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub name: String,
    pub email: String,
    pub date: PrimitiveDateTime,
    pub files: u32,
    pub inserts: u32,
    pub deletes: u32,
}

impl Default for Commit {
    fn default() -> Self {
        // The `time` crate doesn't provide defaults. Fair enough, but if there
        // is a default time, it may as well be the Unix epoch.
        let unix_epoch = datetime!(1970-01-01 0:0:0);

        Self {
            hash: Default::default(),
            name: Default::default(),
            email: Default::default(),
            date: unix_epoch,
            files: Default::default(),
            inserts: Default::default(),
            deletes: Default::default(),
        }
    }
}
