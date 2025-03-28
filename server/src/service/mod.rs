use crate::repository::Repository;
use sqlx::{Pool, Postgres};

mod agents;
mod jobs;
pub struct Service {
    pub repo: Repository,
    pub db: Pool<Postgres>,
}

impl Service {
    pub fn new(db: Pool<Postgres>) -> Self {
        Service {
            repo: Repository {},
            db,
        }
    }
}
