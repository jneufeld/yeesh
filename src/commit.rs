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

impl Default for Commit {
    fn default() -> Self {
        Self {
            hash: Default::default(),
            name: Default::default(),
            email: Default::default(),
            date: Default::default(),
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

    pub fn date(mut self, value: DateTime<FixedOffset>) -> Self {
        self.date = Some(value);
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
