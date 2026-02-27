use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use ring::{
    aead, digest,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};

use crate::{
    app_env,
    utils::constants::{ACCESS_TOKEN_LIFETIME, REFRESH_TOKEN_LIFETIME},
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub sid: String,
    pub iat: i64,
    pub exp: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub sid: String,
    pub iat: i64,
    pub exp: usize,
}

pub fn generate_access_token(user_id: &str, session_id: &str) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ACCESS_TOKEN_LIFETIME);

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

pub fn generate_refresh_token(user_id: &str, session_id: &str) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(REFRESH_TOKEN_LIFETIME);

    let claims = RefreshClaims {
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

pub fn decode_access_token(token: &str) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(app_env!().jwt_secret.as_ref());
    let validation = Validation::default();

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| anyhow::anyhow!("Failed to decode JWT: {:?}", e))
}

pub fn decode_refresh_token(token: &str) -> Result<RefreshClaims> {
    let decoding_key = DecodingKey::from_secret(app_env!().jwt_secret.as_ref());
    let validation = Validation::default();

    decode::<RefreshClaims>(token, &decoding_key, &validation)
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
        .unwrap();

    Ok(hex::encode(&in_out))
}
