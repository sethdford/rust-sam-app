use aws_sdk_dynamodb::{Client, Error};
use aws_sdk_dynamodb::model::AttributeValue;
use std::collections::HashMap;
use tracing::{info, error};
use crate::models::Item;
use crate::error::AppError;

pub struct DynamoDbRepository {
    client: Client,
    table_name: String,
}

impl DynamoDbRepository {
    pub fn new(config: &aws_config::SdkConfig) -> Self {
        let client = Client::new(config);
        let table_name = std::env::var("TABLE_NAME")
            .unwrap_or_else(|_| "Items".to_string());
        
        Self { client, table_name }
    }
    
    pub async fn create_item(&self, item: &Item) -> Result<(), Error> {
        let mut item_attributes = HashMap::new();
        item_attributes.insert("id".to_string(), AttributeValue::S(item.id.clone()));
        item_attributes.insert("name".to_string(), AttributeValue::S(item.name.clone()));
        
        if let Some(desc) = &item.description {
            item_attributes.insert("description".to_string(), AttributeValue::S(desc.clone()));
        }
        
        item_attributes.insert("created_at".to_string(), 
            AttributeValue::S(item.created_at.to_rfc3339()));
        
        info!("Creating item with ID: {}", item.id);
        
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item_attributes))
            .send()
            .await?;
            
        Ok(())
    }
    
    pub async fn get_item(&self, id: &str) -> Result<Option<Item>, Error> {
        info!("Getting item with ID: {}", id);
        
        let response = self.client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await?;
            
        if let Some(item) = response.item {
            let id = item.get("id").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string();
            let name = item.get("name").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string();
            let description = item.get("description").and_then(|v| v.as_s().ok()).map(|s| s.to_string());
            
            let created_at = item.get("created_at")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now);
            
            Ok(Some(Item {
                id,
                name,
                description,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn list_items(&self) -> Result<Vec<Item>, Error> {
        info!("Listing all items");
        
        let response = self.client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await?;
            
        let items = response.items().unwrap_or_default();
        
        let result: Vec<Item> = items
            .iter()
            .filter_map(|item| {
                let id = item.get("id")?.as_s().ok()?;
                let name = item.get("name")?.as_s().ok()?;
                let description = item.get("description").and_then(|v| v.as_s().ok()).cloned();
                
                let created_at = item.get("created_at")
                    .and_then(|v| v.as_s().ok())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now);
                
                Some(Item {
                    id: id.clone(),
                    name: name.clone(),
                    description,
                    created_at,
                })
            })
            .collect();
            
        Ok(result)
    }
    
    pub async fn delete_item(&self, id: &str) -> Result<(), Error> {
        info!("Deleting item with ID: {}", id);
        
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await?;
            
        Ok(())
    }
} 