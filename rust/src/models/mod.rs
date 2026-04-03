use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub balance: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSession {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub last_bet: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
