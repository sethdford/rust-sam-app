#[cfg(test)]
mod tests {
    use crate::models::{Item, ItemEvent, ItemEventType};
    use chrono::{DateTime, Utc};

    #[test]
    fn test_item_serialization() {
        let item = Item {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            created_at: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let serialized = serde_json::to_string(&item).unwrap();
        let deserialized: Item = serde_json::from_str(&serialized).unwrap();

        assert_eq!(item.id, deserialized.id);
        assert_eq!(item.name, deserialized.name);
        assert_eq!(item.description, deserialized.description);
        assert_eq!(item.created_at, deserialized.created_at);
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

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ItemEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ItemEventType::Created, deserialized.event_type);
        assert_eq!(item.id, deserialized.item.id);
        assert_eq!(item.name, deserialized.item.name);
        assert_eq!(item.description, deserialized.item.description);
        assert_eq!(item.created_at, deserialized.item.created_at);
        assert_eq!(event.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_item_event_type_serialization() {
        let event_types = vec![
            ItemEventType::Created,
            ItemEventType::Updated,
            ItemEventType::Deleted,
        ];

        for event_type in event_types {
            let serialized = serde_json::to_string(&event_type).unwrap();
            let deserialized: ItemEventType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(event_type, deserialized);
        }
    }
} 