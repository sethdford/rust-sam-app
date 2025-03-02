use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    #[serde(default = "generate_id")]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub status_code: u16,
    pub body: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemEvent {
    pub event_type: ItemEventType,
    pub item: Item,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemEventType {
    Created,
    Updated,
    Deleted,
} 