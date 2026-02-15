use anyhow::Result;
use ring::{
    aead,
    rand::{SecureRandom, SystemRandom},
};

pub fn generate_token(user_id: &str) -> Result<String> {
    let rng = SystemRandom::new();
    let mut key = [0u8; 32];
    rng.fill(&mut key)
        .map_err(|e| anyhow::anyhow!("Failed to generate key: {:?}", e))?;

    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce)
        .map_err(|e| anyhow::anyhow!("Failed to generate nonce: {:?}", e))?;

    let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key)
        .map_err(|e| anyhow::anyhow!("Failed to create unbound key: {:?}", e))?;
    let sealing_key = aead::LessSafeKey::new(key);

    let mut in_out = format!("token:{}", user_id).as_bytes().to_vec();
    in_out.extend_from_slice(&[0u8; 16]); // space for tag

    sealing_key
        .seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::empty(),
            &mut in_out,
        )
        .unwrap();

    Ok(hex::encode(&in_out))
}
