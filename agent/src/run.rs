use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305,
};
use rand::{rngs::OsRng, RngCore};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use signature::SignerMut;
use std::convert::TryFrom;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use x25519_dalek::x25519;
use zeroize::Zeroize;

use crate::{
    config::{self, Config},
    error::Error,
    init::Response,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentJob {
    pub id: Uuid,
    pub encrypted_job: Vec<u8>,
    pub ephemeral_public_key: [u8; 32], //X25519_PUBLIC_KEY_SIZE
    pub nonce: [u8; 24],                //XCHACHA20_POLY1305_NONCE_SIZE
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobPayload {
    pub command: String,
    pub args: Vec<String>,
    pub result_ephemeral_public_key: [u8; 32], //X25519_PUBLIC_KEY_SIZE
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobResult {
    pub output: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateJobResult {
    pub job_id: Uuid,
    pub encrypted_result: Vec<u8>,
    pub ephemeral_public_key: [u8; 32], // XCHACHA20_POLY1305_NONCE_SIZE 32
    pub nonce: [u8; 24], // XCHACHA20_POLY1305_NONCE_SIZE 24
    pub signature: Vec<u8>,
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
            }
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

        let job_result = match encrypt_and_sign_result(
            &config, 
            job_id, 
            output, 
            job.result_ephemeral_public_key,
        ) {
            Ok(result) => result,
            Err(err) => { 
                println!("{err}");
                sleep(sleep_time).await;
                continue;
            }
        };

        match post_result(&client, &job_result_url, job_result).await {
            Ok(_) => {},
            Err(err) => { 
                println!("{err}"); 
                sleep(sleep_time).await;
                continue;
            },
        };
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

fn decrypt_and_verify_job(config: &Config, job: AgentJob) -> Result<(Uuid, JobPayload), Error> {
    // verify input
    if job.signature.len() != 64 {
        //ED25519_SIGNATURE_SIZE
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


fn encrypt_and_sign_result(
    config: &Config,
    job_id: Uuid,
    output: String,
    result_ephemeral_public_key: [u8; 32],
) -> Result<UpdateJobResult, Error>{
    let mut csprng = OsRng;

    // generate ephemeral keypair for job result encryption
    let mut ephemeral_private_key = [0u8; 32]; //X25519_PRIVATE_KEY_SIZE 32
    csprng.fill_bytes(&mut ephemeral_private_key);
    let ephemeral_public_key = x25519(
        ephemeral_private_key.clone(),
        x25519_dalek::X25519_BASEPOINT_BYTES,
    );

    // key exchange for job result encryption
    let mut shared_secret = x25519(ephemeral_private_key, result_ephemeral_public_key);

    // generate nonce
    let mut nonce = [0u8; 24]; // XCHACHA20_POLY1305_NONCE_SIZE 24
    csprng.fill_bytes(&mut nonce);

    // derive key
    let mut kdf = Blake2bVar::new(32).unwrap(); //XCHACHA20_POLY1305_KEY_SIZE 32
    kdf.update(&shared_secret);
    kdf.update(&nonce);
    let mut key = kdf.finalize_boxed();

    // serialize job result
    let result_payload = JobResult { output };
    let encrypted_result_json = serde_json::to_vec(&result_payload)?;

    // encrypt job result
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    let encrypted_result = cipher.encrypt(&nonce.into(), encrypted_result_json.as_ref())?;

    shared_secret.zeroize();
    key.zeroize();

    // sign job_id, agent_id, encrypted_job, ephemeral_public_key, nonce
    let mut buffer_to_sign = job_id.as_bytes().to_vec();
    buffer_to_sign.append(&mut config.agent_id.as_bytes().to_vec());
    buffer_to_sign.append(&mut encrypted_result.clone());
    buffer_to_sign.append(&mut ephemeral_public_key.to_vec());
    buffer_to_sign.append(&mut nonce.to_vec());

    let mut signing: ed25519_dalek::SigningKey = (&config.signing_private_key).into();
    let signature = signing.sign(&buffer_to_sign);

    Ok(UpdateJobResult {
        job_id,
        encrypted_result,
        ephemeral_public_key,
        nonce,
        signature: signature.to_bytes().to_vec(),
    })
}