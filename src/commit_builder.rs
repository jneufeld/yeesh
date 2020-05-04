use crate::commit::Commit;

pub struct CommitBuilder {
    commit: Commit
}

impl CommitBuilder {
    pub fn new() -> CommitBuilder {
        let commit = Commit {
            hash: "".to_string(),
            name: "".to_string(),
            email: "".to_string(),
            date: "".to_string(),
            files: 0,
            inserts: 0,
            deletes: 0
        };

        CommitBuilder { commit }
    }

    pub fn hash(mut self, hash: String) -> CommitBuilder {
        self.commit.hash = hash;
        self
    }

    pub fn name(mut self, name: String) -> CommitBuilder {
        self.commit.name = name;
        self
    }

    pub fn email(mut self, email: String) -> CommitBuilder {
        self.commit.email = email;
        self
    }

    pub fn date(mut self, date: String) -> CommitBuilder {
        self.commit.date = date;
        self
    }

    pub fn files(mut self, files: u32) -> CommitBuilder {
        self.commit.files = files;
        self
    }

    pub fn inserts(mut self, inserts: u32) -> CommitBuilder {
        self.commit.inserts = inserts;
        self
    }

    pub fn deletes(mut self, deletes: u32) -> CommitBuilder {
        self.commit.deletes = deletes;
        self
    }

    pub fn build(self) -> Commit {
        self.commit
    }
}
