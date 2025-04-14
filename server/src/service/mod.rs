use crate::{config::Config, repository::Repository};
use sqlx::{Pool, Postgres};

mod agents;
mod jobs;

pub const ENCRYPTED_JOB_MAX_SIZE: usize = 512_000; // 512k
pub const ENCRYPTED_JOB_RESULT_MAX_SIZE: usize = 2_000_000; // 2MB

pub struct Service {
    pub repo: Repository,
    pub db: Pool<Postgres>,
    pub config: Config,
}

impl Service {
    pub fn new(db: Pool<Postgres>, config: Config) -> Self {
        Service {
            repo: Repository {},
            db,
            config,
        }
    }
}
