use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305,
};
use ed25519_dalek::ed25519::signature::SignerMut;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::thread::sleep;
use uuid::Uuid;
use x25519_dalek::x25519;
use zeroize::{self, Zeroize};

use crate::api::Job;
use crate::{
    api::{Api, CreateJob, JobResult},
    config::Config,
    error::Error,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobPayload {
    pub command: String,
    pub args: Vec<String>,
    pub result_ephemeral_public_key: [u8; 32], //X25519_PUBLIC_KEY_SIZE
}

pub async fn agent_exec(
    api: &Api,
    agent_id: &String,
    command: &String,
    config: Config,
) -> Result<(), Error> {
    let agent_id = Uuid::parse_str(agent_id.as_str())?;

    let command = command.trim();
    let mut command_with_args: Vec<String> = command
        .split_whitespace()
        .into_iter()
        .map(|s| s.to_owned())
        .collect();
    if command_with_args.is_empty() {
        return Err(Error::InvalidArgument("Command is not valid".to_string()));
    }

    let command = command_with_args.remove(0);
    let args = command_with_args;

    // get agent's info
    let agent = api.get_agent(agent_id).await?;
    // println!("{:?}", agent);
    let signing_public_key: &[u8; 32] = &agent
        .signing_public_key
        .as_slice()
        .try_into()
        .expect("Key is not valid");
    let agent_signing_public_key = ed25519_dalek::VerifyingKey::from_bytes(signing_public_key)?;

    let public_prekey: &[u8; 32] = &agent
        .public_prekey
        .as_slice()
        .try_into()
        .expect("Key is not valid");

    // encrypt job
    let (createjob, job_ephemeral_private_key) = encrypt_and_sign_job(
        &config,
        command,
        args,
        agent.id,
        public_prekey.clone(),
        &agent.public_prekey_signature,
        &agent_signing_public_key,
    )?;

    let job_info = api.post_create_job(createjob).await?;
    // println!("{:?}", job_info);
    //sleep(Duration::from_secs(1));
    
    loop {
        if let Some(job) = api.get_job_result(job_info.id).await? {
            // decrypt job result
            let job_result = decrypt_and_verify_job_result(
                job,
                job_ephemeral_private_key,
                &agent_signing_public_key,
            )?;
            println!("{}", job_result);
            break;
        }
        sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn encrypt_and_sign_job(
    config: &Config,
    command: String,
    args: Vec<String>,
    agent_id: Uuid,
    agent_public_prekey: [u8; 32], //X25519_PUBLIC_KEY_SIZE 32
    agent_public_prekey_signature: &[u8],
    agent_signing_public_key: &ed25519_dalek::VerifyingKey,
) -> Result<(CreateJob, [u8; 32]), Error> {
    // X25519_PRIVATE_KEY_SIZE 32
    if agent_public_prekey_signature.len() != 64 {
        // ED25519_SIGNATURE_SIZE 64
        return Err(Error::Internal(
            "Agent's prekey signature size is not valid".to_string(),
        ));
    }

    // verify agent's prekey
    let agent_public_prekey_buffer = agent_public_prekey.to_vec();
    let signature = ed25519_dalek::Signature::try_from(&agent_public_prekey_signature[0..64])?;
    if agent_signing_public_key
        .verify_strict(&agent_public_prekey_buffer, &signature)
        .is_err()
    {
        return Err(Error::Internal(
            "Agent's prekey Signature is not valid".to_string(),
        ));
    }

    let mut csprng = OsRng;

    // generate ephemeral keypair for job encryption
    let mut job_ephemeral_private_key = [0u8; 32]; //X25519_PRIVATE_KEY_SIZE 32
    csprng.fill_bytes(&mut job_ephemeral_private_key);
    let job_ephemeral_public_key = x25519(
        job_ephemeral_private_key.clone(),
        x25519_dalek::X25519_BASEPOINT_BYTES,
    );

    // generate ephemeral keypair for job result encryption
    let mut job_result_ephemeral_private_key = [0u8; 32];
    csprng.fill_bytes(&mut job_result_ephemeral_private_key);
    let job_result_ephemeral_public_key = x25519(
        job_result_ephemeral_private_key.clone(),
        x25519_dalek::X25519_BASEPOINT_BYTES,
    );

    // key exchange for job encryption
    let mut shared_secret = x25519(job_ephemeral_private_key, agent_public_prekey);

    // generate nonce
    let mut nonce = [0u8; 24]; // XCHACHA20_POLY1305_NONCE_SIZE 24
    csprng.fill_bytes(&mut nonce);

    // derive key
    let mut kdf = Blake2bVar::new(32).unwrap(); //XCHACHA20_POLY1305_KEY_SIZE 32
    kdf.update(&shared_secret);
    kdf.update(&nonce);
    let mut key = kdf.finalize_boxed();

    // serialize job
    let encrypted_job_payload = JobPayload {
        command,
        args,
        result_ephemeral_public_key: job_result_ephemeral_public_key,
    };
    let encrypted_job_json = serde_json::to_vec(&encrypted_job_payload)?;

    // encrypt job
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    let encrypted_job = cipher.encrypt(&nonce.into(), encrypted_job_json.as_ref())?;

    shared_secret.zeroize();
    key.zeroize();

    // other input data
    let job_id = Uuid::new_v4();

    // sign job_id, agent_id, encrypted_job, ephemeral_public_key, nonce
    let mut buffer_to_sign = job_id.as_bytes().to_vec();
    buffer_to_sign.append(&mut agent_id.as_bytes().to_vec());
    buffer_to_sign.append(&mut encrypted_job.clone());
    buffer_to_sign.append(&mut job_ephemeral_public_key.to_vec());
    buffer_to_sign.append(&mut nonce.to_vec());

    let mut signing: ed25519_dalek::SigningKey = (&config.signing_private_key).into();
    let signature = signing.sign(&buffer_to_sign);
    Ok((
        CreateJob {
            id: job_id,
            agent_id,
            encrypted_job,
            ephemeral_public_key: job_ephemeral_public_key,
            nonce,
            signature: signature.to_bytes().to_vec(),
        },
        job_result_ephemeral_private_key,
    ))
}

fn decrypt_and_verify_job_result(
    job: Job,
    job_ephemeral_private_key: [u8; 32],
    agent_signing_public_key: &ed25519_dalek::VerifyingKey,
) -> Result<String, Error> {
    // verify job_id, agent_id, encrypted_job_result, result_ephemeral_public_key, result_nonce
    let encrypted_job_result = job
        .encrypted_result
        .ok_or(Error::Internal("Job's result is missing".to_string()))?;

    let result_ephemeral_public_key: [u8; 32] = job
        .result_ephemeral_public_key
        .ok_or(Error::Internal("Job's result ephemeral public key is missing".to_string()))?
        .as_slice()
        .try_into()
        .expect("not valid");

    let result_nonce: [u8; 24] = job
        .result_nonce
        .ok_or(Error::Internal("Job's result nonce is missing".to_string()))?
        .as_slice()
        .try_into()
        .expect("not valid");

    let mut buffer_to_verify = job.id.as_bytes().to_vec();
    buffer_to_verify.append(&mut job.agent_id.as_bytes().to_vec());
    buffer_to_verify.append(&mut encrypted_job_result.clone());
    buffer_to_verify.append(&mut result_ephemeral_public_key.to_vec());
    buffer_to_verify.append(&mut result_nonce.to_vec());

    let result_signature = job.result_signature.ok_or(Error::Internal(
        "Job's result signature is missing".to_string(),
    ))?;
    if result_signature.len() != 64 {
        return Err(Error::Internal(
            "Job's result signature size is not valid".to_string(),
        ));
    }

    let signature = ed25519_dalek::Signature::try_from(&result_signature[0..64])?;
    if agent_signing_public_key
        .verify_strict(&buffer_to_verify, &signature)
        .is_err()
    {
        return Err(Error::Internal(
            "Agent's prekey Signature is not valid".to_string(),
        ));
    }

    // key exchange for job result decryption
    let mut shared_secret = x25519(job_ephemeral_private_key, result_ephemeral_public_key);

    // derive key
    let mut kdf = Blake2bVar::new(32).unwrap(); //XCHACHA20_POLY1305_KEY_SIZE 32
    kdf.update(&shared_secret);
    kdf.update(&result_nonce);
    let mut key = kdf.finalize_boxed();

    // decrypt job result
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    let decrypted_job_bytes =
        cipher.decrypt(&result_nonce.into(), encrypted_job_result.as_ref())?;

    shared_secret.zeroize();
    key.zeroize();

    // deserialize job result
    let job_result: JobResult = serde_json::from_slice(&decrypted_job_bytes)?;

    Ok(job_result.output)
}