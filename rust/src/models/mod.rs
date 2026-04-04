use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub external_id: String, // Сюда будем сохранять значение из Bearer token (например, "1")
    pub balance: Decimal,
    pub free_spins_left: i32,
    pub is_bonus_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSession {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub last_bet: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}