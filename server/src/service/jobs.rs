use chrono::Utc;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{entities::Job, error::Error, share};

use super::Service;

impl Service {
    pub async fn list_jobs(&self) -> Result<Vec<Job>, Error> {
        self.repo.find_all_jobs(&self.db).await
    }

    pub async fn create_job(&self, input: share::CreateJob) -> Result<Job, Error> {
        // validate input
        if input.encrypted_job.len() > super::ENCRYPTED_JOB_MAX_SIZE {
            return Err(Error::InvalidArgument("Job is too large".to_string()));
        }

        if input.signature.len() != 64 {
            // ED25519_SIGNATURE_SIZE 64
            return Err(Error::InvalidArgument(
                "Signature size is not valid".to_string(),
            ));
        }

        let mut job_buffer = input.id.as_bytes().to_vec();
        job_buffer.append(&mut input.agent_id.as_bytes().to_vec());
        job_buffer.append(&mut input.encrypted_job.clone());
        job_buffer.append(&mut input.ephemeral_public_key.to_vec());
        job_buffer.append(&mut input.nonce.to_vec());

        let signature = ed25519_dalek::Signature::try_from(&input.signature[0..64])?;

        if !self
            .config
            .client_signing_public_key
            .verify_strict(&job_buffer, &signature)
            .is_ok()
        {
            return Err(Error::InvalidArgument("Signature is not valid".to_string()));
        }

        let new_job = Job {
            id: input.id,
            agent_id: input.agent_id,
            encrypted_job: input.encrypted_job,
            ephemeral_public_key: input.ephemeral_public_key.to_vec(),
            nonce: input.nonce.to_vec(),
            signature: input.signature,
            encrypted_result: None,
            result_ephemeral_public_key: None,
            result_nonce: None,
            result_signature: None,
        };

        self.repo.create_job(&self.db, &new_job).await?;

        Ok(new_job)
    }

    pub async fn get_agent_job(&self, agent_id: Uuid) -> Result<Option<Job>, Error> {
        let mut agent = self.repo.find_agent_by_id(&self.db, agent_id).await?;

        agent.last_seen_at = Utc::now();
        // ignore result as an error is not important
        let _ = self.repo.update_agent(&self.db, &agent).await;

        match self.repo.find_job_for_agent(&self.db, agent_id).await {
            Ok(job) => Ok(Some(job)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn list_job_result(&self, job_id: Uuid) -> Result<Job, Error> {
        self.repo.find_job_by_id(&self.db, job_id).await
    }

    // pub async fn update_job_result(&self, job_result: share::UpdateJobResult) -> Result<(), Error> {
    //     let mut job = self
    //         .repo
    //         .find_job_by_id(&self.db, job_result.job_id)
    //         .await?;

    //     job.executed_at = Some(Utc::now());
    //     job.output = Some(job_result.output);
    //     self.repo.update_job(&self.db, &job).await
    // }
}
