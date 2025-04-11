use std::process::Command;
use std::time::Duration;
use reqwest::Client;
use tokio::time::sleep;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use ed25519_dalek::Verifier;
use rand::RngCore;
use std::convert::TryFrom;
use x25519_dalek::x25519;
use zeroize::Zeroize;
use blake2::Blake2bVar;
use blake2::digest::{Update, VariableOutput}; 
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305
};


use crate::{
    init::Response,
    config::{self, Config}, 
    error::Error
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentJob {
    pub id: Uuid,
    pub encrypted_job: Vec<u8>,
    pub ephemeral_public_key: [u8; 32],//X25519_PUBLIC_KEY_SIZE
    pub nonce: [u8; 24],//XCHACHA20_POLY1305_NONCE_SIZE
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobPayload {
    pub command: String,
    pub args: Vec<String>,
    pub result_ephemeral_public_key: [u8; 32], //X25519_PUBLIC_KEY_SIZE
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateJobResult {
    pub job_id: Uuid,
    pub output: String,
}

pub async fn run(client: &Client, config: Config) -> ! {
    let sleep_time = Duration::from_secs(1);
    let job_url = format!("{}/api/jobs/{}", config::SERVER_URL, config.agent_id);
    let job_result_url = format!("{}/api/jobs/result", config::SERVER_URL);

    loop {
       let encrypted_job = match get_job(&client, &job_url).await { 
            Ok(encrypted_job) => encrypted_job,
            Err(err) => {
                println!("{err}");
                sleep(sleep_time).await;
                continue;
            },
        };

        let (job_id, job) = match decrypt_and_verify_job(&config, encrypted_job) {
            Ok(res) => res,
            Err(err) => {
                println!("{err}");
                sleep(sleep_time).await;
                continue;
            }
        };

        let output = executed_command(job.command, job.args);

        let job_result = UpdateJobResult {
            job_id: job_id,
            output
        };
        match post_result(&client, &job_result_url, job_result).await {
            Ok(_) => {},
            Err(err) => { println!("{err}"); },
        };
        sleep(sleep_time).await;
        continue;
    }
}

async fn get_job(client: &Client, job_url: &String) -> Result<AgentJob, Error> {
    
    let resp = client
        .get(job_url)
        .send()
        .await?
        .json::<Response<AgentJob>>()
        .await?;

    let encrypted_job = match resp.data {
        Some(encrypted_job) => Ok(encrypted_job),
        None => Err(Error::Internal("job data is null".to_string())),
    }?;

    Ok(encrypted_job)
}

fn executed_command(command: String, args: Vec<String>) -> String {
    let mut ret = String::new();
    let output = match Command::new(command).args(&args).output() {
        Ok(output) => output,
        _ => return ret,
    };

    ret = match String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        _ => return ret,
    };

    ret
}

async fn post_result(
    client: &Client,
    job_result_url: &String, 
    job_result: UpdateJobResult,
) -> Result<bool, Error> {
    let resp = client
        .post(job_result_url)
        .json(&job_result)
        .send()
        .await?
        .json::<Response<bool>>()
        .await?;

    match resp.data {
        Some(ok) => Ok(ok),
        None => Err(Error::Internal("job data is null".to_string())),
    }
}


fn decrypt_and_verify_job(
    config: &Config, 
    job: AgentJob
) -> Result<(Uuid, JobPayload), Error> {
    // verify input
    if job.signature.len() != 64 { //ED25519_SIGNATURE_SIZE
        return Err(Error::Internal(
            "Job's signature size is not valid".to_string(),
        ));
    }

    // verify job_id, agent_id, encrypted_job, ephemeral_public_key, nonce
    let mut buffer_to_verify = job.id.as_bytes().to_vec();
    buffer_to_verify.append(&mut config.agent_id.as_bytes().to_vec());
    buffer_to_verify.append(&mut job.encrypted_job.clone());
    buffer_to_verify.append(&mut job.ephemeral_public_key.to_vec());
    buffer_to_verify.append(&mut job.nonce.to_vec());

    let signature = ed25519_dalek::Signature::try_from(&job.signature[0..64])?;
    if config
        .client_signing_public_key
        .verify_strict(&buffer_to_verify, &signature)
        .is_err()
    {
        return Err(Error::Internal(
            "Agent's prekey Signature is not valid".to_string(),
        ));
    }

    // key exchange
    let mut shared_secret = x25519(config.private_prekey, job.ephemeral_public_key);

    // derive key
    let mut kdf = Blake2bVar::new(32).unwrap();
    kdf.update(&shared_secret);
    kdf.update(&job.nonce);
    let mut key = kdf.finalize_boxed();

    // decrypt job
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    let decrypted_job_bytes = cipher.decrypt(&job.nonce.into(), job.encrypted_job.as_ref())?;

    shared_secret.zeroize();
    key.zeroize();

    // deserialize job
    let job_payload: JobPayload = serde_json::from_slice(&decrypted_job_bytes)?;

    Ok((job.id, job_payload))
}