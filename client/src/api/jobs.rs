use crate::error::Error;
use super::{Api, CreateJob, Job, Response};


impl Api {
    pub async fn get_list_jobs(&self) -> Result<Vec<Job>, Error>{
        let list_agents_url = format!("{}/api/jobs", self.server_url);
        let resp = self.client
            .get(list_agents_url)
            .send()
            .await?
            .json::<Response<Vec<Job>>>()
            .await?;

        let jobs = resp.data.unwrap();

        Ok(jobs)
    }

    pub async  fn post_create_job(&self, createjob: CreateJob) -> Result<Job, Error>{
        let list_agents_url = format!("{}/api/jobs", self.server_url);

        let resp = self.client
            .post(list_agents_url)
            .json(&createjob)
            .send()
            .await?
            .json::<Response<Job>>()
            .await?;

            let job_info = resp.data.unwrap();
            
        Ok(job_info)
    }
}
