use uuid::Uuid;

use super::{Api, CreateJob, Job, Response};
use crate::error::Error;

impl Api {
    pub async fn get_list_jobs(&self) -> Result<Vec<Job>, Error> {
        let list_jobs_url = format!("{}/api/jobs", self.server_url);
        let resp = self
            .client
            .get(list_jobs_url)
            .send()
            .await?
            .json::<Response<Vec<Job>>>()
            .await?;

        let jobs = resp.data.unwrap();

        Ok(jobs)
    }

    pub async fn post_create_job(&self, createjob: CreateJob) -> Result<Job, Error> {
        let post_job_url = format!("{}/api/jobs", self.server_url);

        let resp = self
            .client
            .post(post_job_url)
            .json(&createjob)
            .send()
            .await?
            .json::<Response<Job>>()
            .await?;

        let job_info = resp.data.unwrap();

        Ok(job_info)
    }

    pub async fn get_job_result(&self, job_id: Uuid) -> Result<Option<Job>, Error> {
        let list_job_url = format!("{}/api/jobs/result/{}", self.server_url, job_id);

        let resp = self.client
            .get(list_job_url)
            .send()
            .await?
            .json::<Response<Job>>()
            .await?;

            let job = resp.data;

        Ok(job)
    }
}
