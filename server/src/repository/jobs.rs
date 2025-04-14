use log::error;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::entities::Job;
use crate::error::Error;

use super::Repository;

impl Repository {
    pub async fn create_job(&self, db: &Pool<Postgres>, job: &Job) -> Result<(), Error> {
        const QUERY: &str = "INSERT INTO jobs
            (id, encrypted_job, ephemeral_public_key, nonce, signature, agent_id)
            VALUES ($1, $2, $3, $4, $5, $6)";

        match sqlx::query(QUERY)
            .bind(job.id)
            .bind(&job.encrypted_job)
            .bind(&job.ephemeral_public_key)
            .bind(&job.nonce)
            .bind(&job.signature)
            .bind(job.agent_id)
            .execute(db)
            .await
        {
            Err(err) => {
                error!("create_job: Inserting job: {}", &err);
                Err(err.into())
            }
            Ok(_) => Ok(()),
        }
    }

    pub async fn find_all_jobs(&self, db: &Pool<Postgres>) -> Result<Vec<Job>, Error> {
        const QUERY: &str = "SELECT * FROM jobs ORDER BY agent_id";

        match sqlx::query_as::<_, Job>(QUERY).fetch_all(db).await {
            Err(err) => {
                error!("find_all_jobs: finding jobs: {}", &err);
                Err(err.into())
            }
            Ok(res) => Ok(res),
        }
    }

    pub async fn find_job_for_agent(
        &self,
        db: &Pool<Postgres>,
        agent_id: Uuid,
    ) -> Result<Job, Error> {
        const QUERY: &str = "SELECT * FROM jobs
            WHERE agent_id = $1 AND encrypted_result IS NULL
            LIMIT 1";

        match sqlx::query_as::<_, Job>(QUERY)
            .bind(agent_id)
            .fetch_optional(db)
            .await
        {
            Err(err) => {
                error!("find_job_where_output_is_null: finding job: {}", &err);
                Err(err.into())
            }
            Ok(None) => Err(Error::NotFound("Job not found.".to_string())),
            Ok(Some(res)) => Ok(res),
        }
    }

    pub async fn find_job_by_id(&self, db: &Pool<Postgres>, job_id: Uuid) -> Result<Job, Error> {
        const QUERY: &str = "SELECT * FROM jobs WHERE id = $1 LIMIT 1";

        match sqlx::query_as::<_, Job>(QUERY)
            .bind(job_id)
            .fetch_optional(db)
            .await
        {
            Err(err) => {
                error!("find_job_where_output_is_null: finding job: {}", &err);
                Err(err.into())
            }
            Ok(None) => Err(Error::NotFound("Job not found.".to_string())),
            Ok(Some(res)) => Ok(res),
        }
    }

    // pub async fn update_job(&self, db: &Pool<Postgres>, job: &Job) -> Result<(), Error> {
    //     const QUERY: &str = "UPDATE jobs
    //         SET executed_at = $1, output = $2
    //         WHERE id = $3";

    //     match sqlx::query(QUERY)
    //         .bind(job.executed_at)
    //         .bind(&job.output)
    //         .bind(job.id)
    //         .execute(db)
    //         .await
    //     {
    //         Err(err) => {
    //             error!("update_job: updating job: {}", &err);
    //             Err(err.into())
    //         }
    //         Ok(_) => Ok(()),
    //     }
    // }
}
