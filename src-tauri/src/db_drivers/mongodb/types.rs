// BSON to DataValue type conversion utilities

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::DataValue,
};
use bson::{Bson, Document};

/// Converter for BSON types to our unified DataValue type
pub struct BsonConverter;

impl BsonConverter {
    /// Convert a BSON value to DataValue
    pub fn bson_to_data_value(bson: &Bson) -> DataValue {
        match bson {
            Bson::Double(v) => DataValue::Float(*v),
            Bson::String(v) => DataValue::String(v.clone()),
            Bson::Array(arr) => {
                let values: Vec<DataValue> = arr.iter().map(Self::bson_to_data_value).collect();
                DataValue::Array(values)
            }
            Bson::Document(doc) => {
                // Convert nested documents to JSON
                match serde_json::to_value(doc) {
                    Ok(json_val) => DataValue::Json(json_val),
                    Err(_) => DataValue::String(format!("{:?}", doc)),
                }
            }
            Bson::Boolean(v) => DataValue::Boolean(*v),
            Bson::Null => DataValue::Null,
            Bson::RegularExpression(regex) => {
                DataValue::String(format!("/{}/{}", regex.pattern, regex.options))
            }
            Bson::JavaScriptCode(code) => DataValue::String(code.clone()),
            Bson::JavaScriptCodeWithScope(code_with_scope) => DataValue::String(format!(
                "{} (scope: {:?})",
                code_with_scope.code, code_with_scope.scope
            )),
            Bson::Int32(v) => DataValue::Integer(*v as i64),
            Bson::Int64(v) => DataValue::Integer(*v),
            Bson::Timestamp(ts) => {
                DataValue::String(format!("Timestamp(t: {}, i: {})", ts.time, ts.increment))
            }
            Bson::Binary(bin) => DataValue::Bytes(bin.bytes.clone()),
            Bson::ObjectId(oid) => DataValue::String(oid.to_hex()),
            Bson::DateTime(dt) => {
                // Convert bson::DateTime (milliseconds since epoch) to chrono::DateTime
                let timestamp_millis = dt.timestamp_millis();
                let datetime = chrono::DateTime::from_timestamp_millis(timestamp_millis)
                    .unwrap_or_else(chrono::Utc::now);
                DataValue::DateTime(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
            }
            Bson::Symbol(sym) => DataValue::String(sym.clone()),
            Bson::Decimal128(dec) => DataValue::String(dec.to_string()),
            Bson::Undefined => DataValue::Null,
            Bson::MaxKey => DataValue::String("MaxKey".to_string()),
            Bson::MinKey => DataValue::String("MinKey".to_string()),
            Bson::DbPointer(_ptr) => {
                // DbPointer fields are private, use generic representation
                DataValue::String("DBPointer(...)".to_string())
            }
        }
    }

    /// Convert a BSON document to a row of DataValues
    pub fn document_to_row(doc: &Document) -> Vec<DataValue> {
        doc.iter()
            .map(|(_, value)| Self::bson_to_data_value(value))
            .collect()
    }

    /// Convert DataValue to BSON (for query parameters)
    pub fn data_value_to_bson(value: &DataValue) -> DriverResult<Bson> {
        match value {
            DataValue::Null => Ok(Bson::Null),
            DataValue::Boolean(b) => Ok(Bson::Boolean(*b)),
            DataValue::Integer(i) => Ok(Bson::Int64(*i)),
            DataValue::Float(f) => Ok(Bson::Double(*f)),
            DataValue::String(s) => Ok(Bson::String(s.clone())),
            DataValue::Bytes(b) => Ok(Bson::Binary(bson::Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: b.clone(),
            })),
            DataValue::Date(s) | DataValue::DateTime(s) => {
                // Try to parse ISO 8601 datetime
                match chrono::DateTime::parse_from_rfc3339(s) {
                    Ok(dt) => {
                        // Convert to milliseconds since epoch for bson::DateTime
                        let timestamp_millis = dt.timestamp_millis();
                        Ok(Bson::DateTime(bson::DateTime::from_millis(
                            timestamp_millis,
                        )))
                    }
                    Err(_) => {
                        // Fall back to string if parsing fails
                        Ok(Bson::String(s.clone()))
                    }
                }
            }
            DataValue::Json(j) => {
                // Convert JSON value to BSON document
                bson::to_bson(j)
                    .map_err(|e| DriverError::TypeConversionError(format!("JSON to BSON: {}", e)))
            }
            DataValue::Array(arr) => {
                let bson_array: Result<Vec<Bson>, DriverError> =
                    arr.iter().map(Self::data_value_to_bson).collect();
                Ok(Bson::Array(bson_array?))
            }
        }
    }

    /// Infer MongoDB data type from BSON value
    pub fn infer_bson_type(bson: &Bson) -> String {
        match bson {
            Bson::Double(_) => "double".to_string(),
            Bson::String(_) => "string".to_string(),
            Bson::Array(_) => "array".to_string(),
            Bson::Document(_) => "document".to_string(),
            Bson::Boolean(_) => "bool".to_string(),
            Bson::Null | Bson::Undefined => "null".to_string(),
            Bson::RegularExpression(_) => "regex".to_string(),
            Bson::JavaScriptCode(_) | Bson::JavaScriptCodeWithScope(_) => "javascript".to_string(),
            Bson::Int32(_) => "int".to_string(),
            Bson::Int64(_) => "long".to_string(),
            Bson::Timestamp(_) => "timestamp".to_string(),
            Bson::Binary(_) => "binData".to_string(),
            Bson::ObjectId(_) => "objectId".to_string(),
            Bson::DateTime(_) => "date".to_string(),
            Bson::Symbol(_) => "symbol".to_string(),
            Bson::Decimal128(_) => "decimal".to_string(),
            Bson::MaxKey => "maxKey".to_string(),
            Bson::MinKey => "minKey".to_string(),
            Bson::DbPointer(_) => "dbPointer".to_string(),
        }
    }

    /// Get column names from a BSON document in order
    pub fn get_column_names(doc: &Document) -> Vec<String> {
        doc.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::doc;

    #[test]
    fn test_bson_to_data_value_primitives() {
        assert_eq!(
            BsonConverter::bson_to_data_value(&Bson::String("test".to_string())),
            DataValue::String("test".to_string())
        );

        assert_eq!(
            BsonConverter::bson_to_data_value(&Bson::Int32(42)),
            DataValue::Integer(42)
        );

        assert_eq!(
            BsonConverter::bson_to_data_value(&Bson::Int64(100)),
            DataValue::Integer(100)
        );

        assert_eq!(
            BsonConverter::bson_to_data_value(&Bson::Double(3.14)),
            DataValue::Float(3.14)
        );

        assert_eq!(
            BsonConverter::bson_to_data_value(&Bson::Boolean(true)),
            DataValue::Boolean(true)
        );

        assert_eq!(
            BsonConverter::bson_to_data_value(&Bson::Null),
            DataValue::Null
        );
    }

    #[test]
    fn test_bson_to_data_value_array() {
        let bson_array = Bson::Array(vec![Bson::Int32(1), Bson::Int32(2), Bson::Int32(3)]);

        let result = BsonConverter::bson_to_data_value(&bson_array);
        match result {
            DataValue::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], DataValue::Integer(1));
                assert_eq!(arr[1], DataValue::Integer(2));
                assert_eq!(arr[2], DataValue::Integer(3));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_bson_to_data_value_objectid() {
        let oid = bson::oid::ObjectId::new();
        let bson_oid = Bson::ObjectId(oid);
        let result = BsonConverter::bson_to_data_value(&bson_oid);

        match result {
            DataValue::String(s) => {
                assert_eq!(s, oid.to_hex());
            }
            _ => panic!("Expected String for ObjectId"),
        }
    }

    #[test]
    fn test_data_value_to_bson() {
        // Test primitive types
        let string_val = DataValue::String("test".to_string());
        let bson = BsonConverter::data_value_to_bson(&string_val).unwrap();
        assert_eq!(bson, Bson::String("test".to_string()));

        let int_val = DataValue::Integer(42);
        let bson = BsonConverter::data_value_to_bson(&int_val).unwrap();
        assert_eq!(bson, Bson::Int64(42));

        let bool_val = DataValue::Boolean(true);
        let bson = BsonConverter::data_value_to_bson(&bool_val).unwrap();
        assert_eq!(bson, Bson::Boolean(true));

        let null_val = DataValue::Null;
        let bson = BsonConverter::data_value_to_bson(&null_val).unwrap();
        assert_eq!(bson, Bson::Null);
    }

    #[test]
    fn test_infer_bson_type() {
        assert_eq!(
            BsonConverter::infer_bson_type(&Bson::String("test".to_string())),
            "string"
        );
        assert_eq!(BsonConverter::infer_bson_type(&Bson::Int32(42)), "int");
        assert_eq!(BsonConverter::infer_bson_type(&Bson::Int64(100)), "long");
        assert_eq!(
            BsonConverter::infer_bson_type(&Bson::Double(3.14)),
            "double"
        );
        assert_eq!(BsonConverter::infer_bson_type(&Bson::Boolean(true)), "bool");
        assert_eq!(BsonConverter::infer_bson_type(&Bson::Null), "null");
        assert_eq!(
            BsonConverter::infer_bson_type(&Bson::ObjectId(bson::oid::ObjectId::new())),
            "objectId"
        );
    }

    #[test]
    fn test_document_to_row() {
        let doc = doc! {
            "name": "Alice",
            "age": 30,
            "active": true
        };

        let row = BsonConverter::document_to_row(&doc);
        assert_eq!(row.len(), 3);
    }

    #[test]
    fn test_get_column_names() {
        let doc = doc! {
            "id": 1,
            "name": "Test",
            "value": 42.5
        };

        let columns = BsonConverter::get_column_names(&doc);
        assert_eq!(columns.len(), 3);
        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"value".to_string()));
    }
}
