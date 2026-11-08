use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{domain::error::ApiError, openapi::schemas::HistoryCursor};

#[derive(Deserialize, Serialize)]
struct CursorPayload {
    timestamp: i64,
    id: String,
}

pub fn encode_history_cursor(timestamp: DateTime, id: ObjectId) -> String {
    let payload = CursorPayload {
        timestamp: timestamp.timestamp_millis(),
        id: id.to_hex(),
    };

    let json = serde_json::to_vec(&payload).expect("History cursor serialization should not fail");

    URL_SAFE_NO_PAD.encode(json)
}

pub fn decode_history_cursor(cursor: &str) -> Result<HistoryCursor, ApiError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| ApiError::BadRequest("Invalid history cursor".into()))?;

    let payload: CursorPayload = serde_json::from_slice(&decoded)
        .map_err(|_| ApiError::BadRequest("Invalid history cursor".into()))?;

    let id = ObjectId::parse_str(&payload.id)
        .map_err(|_| ApiError::BadRequest("Invalid history cursor".into()))?;

    let timestamp = DateTime::from_millis(payload.timestamp);

    Ok(HistoryCursor { timestamp, id })
}
