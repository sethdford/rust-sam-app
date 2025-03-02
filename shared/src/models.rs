use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents an item in the system
///
/// This is the core data model for the application. Items are stored in DynamoDB
/// and can be created, retrieved, and deleted through the API.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    /// Unique identifier for the item
    /// 
    /// If not provided when creating an item, a UUID will be automatically generated.
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Name of the item (required)
    pub name: String,
    
    /// Optional description of the item
    pub description: Option<String>,
    
    /// Timestamp when the item was created
    /// 
    /// If not provided when creating an item, the current time will be used.
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Classification level of this item
    /// Options: PUBLIC, INTERNAL, CONFIDENTIAL, RESTRICTED
    #[serde(default = "default_classification")]
    pub classification: String,
}

/// Generates a new UUID string for item IDs
fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Returns the current UTC time for item creation timestamps
fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

/// Returns the default classification level for items
fn default_classification() -> String {
    "INTERNAL".to_string()
}

/// Generic API response wrapper
///
/// This struct is used to wrap API responses with a status code and body.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    /// HTTP status code
    pub status_code: u16,
    
    /// Response body
    pub body: T,
}

/// Error response for API errors
///
/// This struct is used to return error messages to API clients.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub message: String,
}

/// Represents an event related to an item
///
/// Events are sent to SQS when items are created, updated, or deleted.
/// The event processor Lambda consumes these events and performs additional processing.
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemEvent {
    /// Type of event (Created, Updated, Deleted)
    pub event_type: ItemEventType,
    
    /// The item associated with the event
    pub item: Item,
    
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
}

/// Types of events that can occur for items
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ItemEventType {
    /// Item was created
    Created,
    
    /// Item was updated
    Updated,
    
    /// Item was deleted
    Deleted,
}

/// Audit record for tracking changes to items
///
/// This struct is used to maintain an audit trail of all changes to items.
/// It includes information about who made the change, what was changed, and when.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Unique identifier for the audit record
    pub event_id: String,
    
    /// ID of the user who performed the action
    pub user_id: String,
    
    /// Action that was performed (create, update, delete)
    pub action: String,
    
    /// ID of the resource that was affected
    pub resource_id: String,
    
    /// Type of resource that was affected
    pub resource_type: String,
    
    /// Timestamp when the action occurred
    pub timestamp: DateTime<Utc>,
    
    /// Previous state of the resource (for updates and deletes)
    pub previous_state: Option<String>,
    
    /// New state of the resource (for creates and updates)
    pub new_state: Option<String>,
    
    /// ID of the request that triggered the action
    pub request_id: String,
    
    /// Hash of the original request for non-repudiation
    pub hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_item_serialization() {
        let item = Item {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            created_at: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            classification: "INTERNAL".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        let expected = json!({
            "id": "test-id",
            "name": "Test Item",
            "description": "Test Description",
            "created_at": "2023-01-01T00:00:00Z",
            "classification": "INTERNAL"
        });

        assert_eq!(serde_json::from_str::<serde_json::Value>(&json).unwrap(), expected);
    }

    #[test]
    fn test_item_deserialization() {
        let json = r#"{
            "id": "test-id",
            "name": "Test Item",
            "description": "Test Description",
            "created_at": "2023-01-01T00:00:00Z",
            "classification": "INTERNAL"
        }"#;

        let item: Item = serde_json::from_str(json).unwrap();
        
        assert_eq!(item.id, "test-id");
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, Some("Test Description".to_string()));
        assert_eq!(
            item.created_at,
            DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        assert_eq!(item.classification, "INTERNAL");
    }

    #[test]
    fn test_item_default_values() {
        let json = r#"{
            "name": "Test Item"
        }"#;

        let item: Item = serde_json::from_str(json).unwrap();
        
        assert!(!item.id.is_empty()); // ID should be auto-generated
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, None);
        // created_at should be auto-generated and close to now
        let now = Utc::now();
        let diff = now.signed_duration_since(item.created_at);
        assert!(diff.num_seconds() < 10); // Should be within 10 seconds
        assert_eq!(item.classification, "INTERNAL");
    }

    #[test]
    fn test_item_event_serialization() {
        let item = Item {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            created_at: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let event = ItemEvent {
            event_type: ItemEventType::Created,
            item: item.clone(),
            timestamp: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let json = serde_json::to_string(&event).unwrap();
        let expected = json!({
            "event_type": "Created",
            "item": {
                "id": "test-id",
                "name": "Test Item",
                "description": "Test Description",
                "created_at": "2023-01-01T00:00:00Z"
            },
            "timestamp": "2023-01-01T00:00:00Z"
        });

        assert_eq!(serde_json::from_str::<serde_json::Value>(&json).unwrap(), expected);
    }
} 