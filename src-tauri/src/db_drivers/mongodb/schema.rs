// MongoDB schema introspection utilities
// Since MongoDB is schemaless, we infer structure from sample documents

use super::types::BsonConverter;
use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::{Column, Index, Schema, Table},
};
use bson::{doc, Document};
use mongodb::{Client, Database};
use std::collections::{HashMap, HashSet};

/// Schema inspector for MongoDB databases
pub struct MongoSchemaInspector;

impl MongoSchemaInspector {
    /// Get complete schema for a MongoDB database
    /// Samples documents from each collection to infer structure
    pub async fn get_schema(client: &Client, database_name: &str) -> DriverResult<Schema> {
        let db = client.database(database_name);

        // List all collections
        let collections = db.list_collection_names().await.map_err(|e| {
            DriverError::SchemaIntrospectionFailed(format!("Failed to list collections: {}", e))
        })?;

        let mut tables = Vec::new();

        for collection_name in collections {
            match Self::introspect_collection(&db, &collection_name).await {
                Ok(table) => tables.push(table),
                Err(e) => {
                    // Log error but continue with other collections
                    eprintln!(
                        "Warning: Failed to introspect collection {}: {}",
                        collection_name, e
                    );
                }
            }
        }

        Ok(Schema {
            database_name: database_name.to_string(),
            tables,
        })
    }

    /// Introspect a single collection to infer its schema
    /// Samples first 100 documents to determine field types
    async fn introspect_collection(db: &Database, collection_name: &str) -> DriverResult<Table> {
        let collection = db.collection::<Document>(collection_name);

        // Get sample documents (first 100)
        let mut cursor = collection.find(doc! {}).limit(100).await.map_err(|e| {
            DriverError::SchemaIntrospectionFailed(format!("Failed to query collection: {}", e))
        })?;

        let mut field_types: HashMap<String, HashSet<String>> = HashMap::new();
        let mut field_nullable: HashMap<String, bool> = HashMap::new();
        let mut doc_count = 0;

        // Collect field information from sample documents
        use futures::stream::StreamExt;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    doc_count += 1;
                    Self::analyze_document(&doc, &mut field_types, &mut field_nullable, "");
                }
                Err(e) => {
                    eprintln!("Warning: Error reading document: {}", e);
                }
            }
        }

        // Convert field information to columns
        let mut columns = Vec::new();
        let mut field_names: Vec<_> = field_types.keys().cloned().collect();
        field_names.sort(); // Sort for consistent ordering

        for field_name in field_names {
            let types = field_types.get(&field_name).unwrap();
            let is_nullable = field_nullable.get(&field_name).copied().unwrap_or(false);

            // Determine primary data type (most common or first seen)
            let data_type = if types.len() == 1 {
                types.iter().next().unwrap().clone()
            } else {
                // Multiple types - use "mixed" or pick most common
                format!(
                    "mixed({})",
                    types.iter().cloned().collect::<Vec<_>>().join(", ")
                )
            };

            let is_id_field = field_name == "_id";

            columns.push(Column {
                name: field_name.clone(),
                data_type,
                nullable: is_nullable && !is_id_field,
                default_value: None,
                primary_key: is_id_field,
                auto_increment: is_id_field, // _id is auto-generated
            });
        }

        // Get indexes for this collection
        let indexes = Self::get_collection_indexes(&collection).await?;

        Ok(Table {
            name: collection_name.to_string(),
            schema: Some(db.name().to_string()),
            columns,
            indexes,
            foreign_keys: vec![], // MongoDB doesn't enforce foreign keys
            row_count: Some(doc_count),
        })
    }

    /// Analyze a document recursively to extract field types
    fn analyze_document(
        doc: &Document,
        field_types: &mut HashMap<String, HashSet<String>>,
        field_nullable: &mut HashMap<String, bool>,
        prefix: &str,
    ) {
        for (key, value) in doc.iter() {
            let field_name = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            // Track field type
            let bson_type = BsonConverter::infer_bson_type(value);
            field_types
                .entry(field_name.clone())
                .or_default()
                .insert(bson_type.clone());

            // Track if field can be null
            if value.as_null().is_some() {
                field_nullable.insert(field_name.clone(), true);
            }

            // Recursively analyze nested documents (limited depth)
            if let Some(nested_doc) = value.as_document() {
                if prefix.split('.').count() < 2 {
                    // Limit nesting to prevent explosion
                    Self::analyze_document(nested_doc, field_types, field_nullable, &field_name);
                }
            }
        }
    }

    /// Get indexes for a collection
    async fn get_collection_indexes(
        collection: &mongodb::Collection<Document>,
    ) -> DriverResult<Vec<Index>> {
        let mut cursor = collection.list_indexes().await.map_err(|e| {
            DriverError::SchemaIntrospectionFailed(format!("Failed to list indexes: {}", e))
        })?;

        let mut indexes = Vec::new();

        use futures::stream::StreamExt;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(index_model) => {
                    // Extract index name
                    let index_name = index_model
                        .options
                        .as_ref()
                        .and_then(|opts| opts.name.clone())
                        .unwrap_or_else(|| "unnamed".to_string());

                    // Extract indexed columns
                    let mut columns = Vec::new();
                    for (key, _) in index_model.keys.iter() {
                        columns.push(key.clone());
                    }

                    // Check if unique
                    let is_unique = index_model
                        .options
                        .as_ref()
                        .and_then(|opts| opts.unique)
                        .unwrap_or(false);

                    indexes.push(Index {
                        name: index_name,
                        columns,
                        unique: is_unique,
                        index_type: "btree".to_string(), // MongoDB uses B-tree by default
                    });
                }
                Err(e) => {
                    eprintln!("Warning: Error reading index: {}", e);
                }
            }
        }

        Ok(indexes)
    }

    /// List all databases accessible to the client
    pub async fn list_databases(client: &Client) -> DriverResult<Vec<String>> {
        let db_names = client.list_database_names().await.map_err(|e| {
            DriverError::SchemaIntrospectionFailed(format!("Failed to list databases: {}", e))
        })?;

        Ok(db_names)
    }

    /// List all collections in a database
    pub async fn list_collections(
        client: &Client,
        database_name: &str,
    ) -> DriverResult<Vec<String>> {
        let db = client.database(database_name);

        let collections = db.list_collection_names().await.map_err(|e| {
            DriverError::SchemaIntrospectionFailed(format!("Failed to list collections: {}", e))
        })?;

        Ok(collections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::doc;

    #[test]
    fn test_analyze_document_simple() {
        let doc = doc! {
            "name": "Alice",
            "age": 30,
            "active": true
        };

        let mut field_types: HashMap<String, HashSet<String>> = HashMap::new();
        let mut field_nullable: HashMap<String, bool> = HashMap::new();

        MongoSchemaInspector::analyze_document(&doc, &mut field_types, &mut field_nullable, "");

        assert_eq!(field_types.len(), 3);
        assert!(field_types.contains_key("name"));
        assert!(field_types.contains_key("age"));
        assert!(field_types.contains_key("active"));

        assert_eq!(field_types["name"].iter().next().unwrap(), "string");
        assert_eq!(field_types["age"].iter().next().unwrap(), "int");
        assert_eq!(field_types["active"].iter().next().unwrap(), "bool");
    }

    #[test]
    fn test_analyze_document_with_null() {
        let doc = doc! {
            "name": "Bob",
            "email": bson::Bson::Null
        };

        let mut field_types: HashMap<String, HashSet<String>> = HashMap::new();
        let mut field_nullable: HashMap<String, bool> = HashMap::new();

        MongoSchemaInspector::analyze_document(&doc, &mut field_types, &mut field_nullable, "");

        assert!(field_nullable.get("email").copied().unwrap_or(false));
    }

    #[test]
    fn test_analyze_document_nested() {
        let doc = doc! {
            "user": {
                "name": "Charlie",
                "age": 25
            },
            "score": 95.5
        };

        let mut field_types: HashMap<String, HashSet<String>> = HashMap::new();
        let mut field_nullable: HashMap<String, bool> = HashMap::new();

        MongoSchemaInspector::analyze_document(&doc, &mut field_types, &mut field_nullable, "");

        // Check nested fields
        assert!(field_types.contains_key("user"));
        assert!(field_types.contains_key("user.name"));
        assert!(field_types.contains_key("user.age"));
        assert!(field_types.contains_key("score"));
    }

    #[test]
    fn test_analyze_document_mixed_types() {
        let doc1 = doc! { "value": 42 };
        let doc2 = doc! { "value": "text" };

        let mut field_types: HashMap<String, HashSet<String>> = HashMap::new();
        let mut field_nullable: HashMap<String, bool> = HashMap::new();

        MongoSchemaInspector::analyze_document(&doc1, &mut field_types, &mut field_nullable, "");
        MongoSchemaInspector::analyze_document(&doc2, &mut field_types, &mut field_nullable, "");

        // Should have both "int" and "string" types for "value"
        let types = &field_types["value"];
        assert_eq!(types.len(), 2);
        assert!(types.contains("int"));
        assert!(types.contains("string"));
    }
}
