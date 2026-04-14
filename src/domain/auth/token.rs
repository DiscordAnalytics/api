use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use ring::{
    aead, digest,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};

use crate::app_env;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String, // user ID
    pub sid: String, // session ID
    pub iat: i64,    // issued at
    pub exp: usize,  // expiration time
}

pub fn generate_token(user_id: &str, session_id: &str, lifetime: i64) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(lifetime);

    let claims = Claims {
        sub: user_id.to_string(),
        sid: session_id.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp() as usize,
    };

    let encoding_key = EncodingKey::from_secret(app_env!().jwt_secret.as_bytes());
    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| anyhow::anyhow!("Failed to encode JWT: {:?}", e))
}

pub fn hash_refresh_token(token: &str) -> String {
    let hash = digest::digest(&digest::SHA256, token.as_bytes());
    hex::encode(hash.as_ref())
}

pub fn decode_token(token: &str) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(app_env!().jwt_secret.as_ref());
    let validation = Validation::default();

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| anyhow::anyhow!("Failed to decode JWT: {:?}", e))
}

pub fn generate_bot_token(bot_id: &str) -> Result<String> {
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

    let mut in_out = format!("token:{}", bot_id).as_bytes().to_vec();
    in_out.extend_from_slice(&[0u8; 16]);

    sealing_key
        .seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::empty(),
            &mut in_out,
        )
        .map_err(|e| anyhow::anyhow!("Failed to seal token: {:?}", e))?;

    Ok(hex::encode(&in_out))
}
