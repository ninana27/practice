use rand::rngs::OsRng;
use ed25519_dalek::SigningKey;
use base64::{Engine as _, engine::general_purpose};

pub fn generate_keypair() {
    let mut csprng = OsRng;
    // generate signing keypair
    let signing_keypair: SigningKey = SigningKey::generate(&mut csprng);

    let encoded_private_key = general_purpose::STANDARD
        .encode(signing_keypair.to_bytes());
    println!("private key: {}", encoded_private_key);

    let encoded_public_key = general_purpose::STANDARD
        .encode(signing_keypair.verifying_key().to_bytes());
    println!("public key: {}", encoded_public_key);

}
    