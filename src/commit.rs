#[derive(Debug)]
pub struct Commit {
    pub hash: String,
    pub name: String,
    pub email: String,
    pub date: String,
    pub files: u32,
    pub inserts: u32,
    pub deletes: u32,
}
