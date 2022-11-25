use time::{macros::datetime, PrimitiveDateTime};

#[derive(Debug)]
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

impl Commit {
    pub fn hash(mut self, value: &str) -> Self {
        self.hash = String::from(value);
        self
    }

    pub fn name(mut self, value: &str) -> Self {
        self.name = String::from(value);
        self
    }

    pub fn email(mut self, value: &str) -> Self {
        self.email = String::from(value);
        self
    }

    pub fn date(mut self, value: PrimitiveDateTime) -> Self {
        self.date = value;
        self
    }

    pub fn files(mut self, value: u32) -> Self {
        self.files = value;
        self
    }

    pub fn inserts(mut self, value: u32) -> Self {
        self.inserts = value;
        self
    }

    pub fn deletes(mut self, value: u32) -> Self {
        self.deletes = value;
        self
    }
}
