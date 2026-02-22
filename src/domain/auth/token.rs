use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use ring::{
    aead,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};

use crate::app_env;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(user_id: String, exp_hours: u64) -> Self {
        let exp = (Utc::now() + Duration::hours(exp_hours as i64)).timestamp() as usize;

        Self { sub: user_id, exp }
    }
}

pub fn generate_jwt(user_id: &str) -> Result<String> {
    let claims = Claims::new(user_id.to_string(), 24 * 7);

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(app_env!().jwt_secret.as_ref());

    jsonwebtoken::encode(&header, &claims, &encoding_key)
        .map_err(|e| anyhow::anyhow!("Failed to generate JWT: {:?}", e))
}

pub fn decode_jwt(token: &str) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(app_env!().jwt_secret.as_ref());
    let validation = Validation::default();

    jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
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
