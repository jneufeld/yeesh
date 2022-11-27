use time::OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct Author {
    pub name: String,
    pub email: String,
}

impl Author {
    pub fn new(name: String, email: String) -> Author {
        Author { name, email }
    }
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub author: Author,
    pub date: OffsetDateTime,
    pub files: u32,
    pub inserts: u32,
    pub deletes: u32,
}

impl Default for Commit {
    fn default() -> Self {
        let right_now = OffsetDateTime::now_utc();

        Self {
            hash: Default::default(),
            author: Default::default(),
            date: right_now,
            files: Default::default(),
            inserts: Default::default(),
            deletes: Default::default(),
        }
    }
}
