// Cassandra/Scylla type conversion utilities

use scylla::frame::response::result::CqlValue;

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::DataValue,
};

/// Convert Scylla CqlValue to DataValue
pub fn cql_value_to_data_value(value: &CqlValue) -> DriverResult<DataValue> {
    match value {
        CqlValue::Ascii(s) | CqlValue::Text(s) => Ok(DataValue::String(s.clone())),

        CqlValue::Int(i) => Ok(DataValue::Integer(*i as i64)),
        CqlValue::BigInt(i) => Ok(DataValue::Integer(*i)),
        CqlValue::Counter(i) => Ok(DataValue::Integer(i.0)),
        CqlValue::SmallInt(i) => Ok(DataValue::Integer(*i as i64)),
        CqlValue::TinyInt(i) => Ok(DataValue::Integer(*i as i64)),
        CqlValue::Varint(v) => {
            // Try to convert BigInt to i64
            v.as_signed_bytes_be_slice()
                .try_into()
                .map(i64::from_be_bytes)
                .map(DataValue::Integer)
                .or_else(|_| Ok(DataValue::String(format!("{:?}", v))))
        }

        CqlValue::Boolean(b) => Ok(DataValue::Boolean(*b)),

        CqlValue::Float(f) => Ok(DataValue::Float(*f as f64)),
        CqlValue::Double(d) => Ok(DataValue::Float(*d)),
        CqlValue::Decimal(d) => Ok(DataValue::String(format!("{:?}", d))),

        CqlValue::Uuid(uuid) => Ok(DataValue::String(uuid.to_string())),
        CqlValue::Timeuuid(timeuuid) => Ok(DataValue::String(timeuuid.to_string())),

        CqlValue::Inet(ip) => Ok(DataValue::String(ip.to_string())),

        CqlValue::Timestamp(ts) => {
            // Cassandra timestamp is milliseconds since epoch
            let duration = ts.0;
            let datetime = chrono::DateTime::from_timestamp_millis(duration)
                .ok_or_else(|| {
                    DriverError::TypeConversionError(format!("Invalid timestamp: {}", duration))
                })?
                .naive_utc();
            Ok(DataValue::DateTime(
                datetime.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ))
        }

        CqlValue::Date(d) => {
            // Cassandra date is days since Unix epoch (1970-01-01)
            let days = d.0 as i64;
            let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .ok_or_else(|| DriverError::TypeConversionError("Invalid epoch".to_string()))?;
            let date = epoch + chrono::Duration::days(days - 2_i64.pow(31));
            Ok(DataValue::Date(date.format("%Y-%m-%d").to_string()))
        }

        CqlValue::Time(t) => {
            // Cassandra time is nanoseconds since midnight
            let nanos = t.0;
            let seconds = nanos / 1_000_000_000;
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            let secs = seconds % 60;
            Ok(DataValue::String(format!(
                "{:02}:{:02}:{:02}",
                hours, minutes, secs
            )))
        }

        CqlValue::Duration(d) => Ok(DataValue::String(format!(
            "{}m{}d{}ns",
            d.months, d.days, d.nanoseconds
        ))),

        CqlValue::Blob(bytes) => Ok(DataValue::Bytes(bytes.clone())),

        CqlValue::List(list) => {
            let converted: Result<Vec<DataValue>, DriverError> =
                list.iter().map(cql_value_to_data_value).collect();
            Ok(DataValue::Array(converted?))
        }

        CqlValue::Set(set) => {
            let converted: Result<Vec<DataValue>, DriverError> =
                set.iter().map(cql_value_to_data_value).collect();
            Ok(DataValue::Array(converted?))
        }

        CqlValue::Map(map) => {
            // Convert map to JSON object
            let mut json_map = serde_json::Map::new();
            for (key, value) in map {
                let key_str = match cql_value_to_data_value(key)? {
                    DataValue::String(s) => s,
                    other => other.to_string(),
                };
                let value_json = data_value_to_json(&cql_value_to_data_value(value)?);
                json_map.insert(key_str, value_json);
            }
            Ok(DataValue::Json(serde_json::Value::Object(json_map)))
        }

        CqlValue::Tuple(tuple) => {
            let converted: Result<Vec<DataValue>, DriverError> = tuple
                .iter()
                .map(|opt| {
                    opt.as_ref()
                        .map(cql_value_to_data_value)
                        .transpose()
                        .map(|o| o.unwrap_or(DataValue::Null))
                })
                .collect();
            Ok(DataValue::Array(converted?))
        }

        CqlValue::UserDefinedType { fields, .. } => {
            // Convert UDT to JSON object
            let mut json_map = serde_json::Map::new();
            for (name, value) in fields {
                let value_json = if let Some(v) = value {
                    data_value_to_json(&cql_value_to_data_value(v)?)
                } else {
                    serde_json::Value::Null
                };
                json_map.insert(name.clone(), value_json);
            }
            Ok(DataValue::Json(serde_json::Value::Object(json_map)))
        }

        CqlValue::Empty => Ok(DataValue::Null),
    }
}

/// Convert DataValue to JSON for nested structures
fn data_value_to_json(value: &DataValue) -> serde_json::Value {
    match value {
        DataValue::Null => serde_json::Value::Null,
        DataValue::Boolean(b) => serde_json::Value::Bool(*b),
        DataValue::Integer(i) => serde_json::json!(i),
        DataValue::Float(f) => serde_json::json!(f),
        DataValue::String(s) => serde_json::Value::String(s.clone()),
        DataValue::Bytes(b) => {
            use base64::Engine;
            serde_json::json!(base64::engine::general_purpose::STANDARD.encode(b))
        }
        DataValue::Date(d) => serde_json::Value::String(d.clone()),
        DataValue::DateTime(dt) => serde_json::Value::String(dt.clone()),
        DataValue::Json(j) => j.clone(),
        DataValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(data_value_to_json).collect())
        }
    }
}

/// Map Cassandra CQL type name to display string
pub fn cql_type_to_string(type_name: &str) -> String {
    match type_name {
        "org.apache.cassandra.db.marshal.UTF8Type" => "text".to_string(),
        "org.apache.cassandra.db.marshal.AsciiType" => "ascii".to_string(),
        "org.apache.cassandra.db.marshal.Int32Type" => "int".to_string(),
        "org.apache.cassandra.db.marshal.LongType" => "bigint".to_string(),
        "org.apache.cassandra.db.marshal.BooleanType" => "boolean".to_string(),
        "org.apache.cassandra.db.marshal.FloatType" => "float".to_string(),
        "org.apache.cassandra.db.marshal.DoubleType" => "double".to_string(),
        "org.apache.cassandra.db.marshal.UUIDType" => "uuid".to_string(),
        "org.apache.cassandra.db.marshal.TimestampType" => "timestamp".to_string(),
        "org.apache.cassandra.db.marshal.SimpleDateType" => "date".to_string(),
        "org.apache.cassandra.db.marshal.TimeType" => "time".to_string(),
        "org.apache.cassandra.db.marshal.BytesType" => "blob".to_string(),
        _ => type_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla::frame::value::{CqlDate, CqlDuration, CqlTime, CqlTimestamp};
    use uuid::Uuid;

    #[test]
    fn test_cql_value_conversion_text() {
        let value = CqlValue::Text("hello".to_string());
        let result = cql_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::String("hello".to_string()));
    }

    #[test]
    fn test_cql_value_conversion_int() {
        let value = CqlValue::Int(42);
        let result = cql_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::Integer(42));
    }

    #[test]
    fn test_cql_value_conversion_bigint() {
        let value = CqlValue::BigInt(9223372036854775807);
        let result = cql_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::Integer(9223372036854775807));
    }

    #[test]
    fn test_cql_value_conversion_boolean() {
        let value = CqlValue::Boolean(true);
        let result = cql_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::Boolean(true));
    }

    #[test]
    fn test_cql_value_conversion_float() {
        let value = CqlValue::Float(3.14);
        let result = cql_value_to_data_value(&value).unwrap();
        match result {
            DataValue::Float(f) => assert!((f - 3.14).abs() < 0.01),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_cql_value_conversion_uuid() {
        let uuid = Uuid::new_v4();
        let value = CqlValue::Uuid(uuid);
        let result = cql_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::String(uuid.to_string()));
    }

    #[test]
    fn test_cql_value_conversion_list() {
        let value = CqlValue::List(vec![
            CqlValue::Text("item1".to_string()),
            CqlValue::Text("item2".to_string()),
        ]);
        let result = cql_value_to_data_value(&value).unwrap();

        match result {
            DataValue::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], DataValue::String("item1".to_string()));
                assert_eq!(arr[1], DataValue::String("item2".to_string()));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_cql_value_conversion_empty() {
        let value = CqlValue::Empty;
        let result = cql_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_cql_type_to_string() {
        assert_eq!(
            cql_type_to_string("org.apache.cassandra.db.marshal.UTF8Type"),
            "text"
        );
        assert_eq!(
            cql_type_to_string("org.apache.cassandra.db.marshal.Int32Type"),
            "int"
        );
        assert_eq!(
            cql_type_to_string("org.apache.cassandra.db.marshal.BooleanType"),
            "boolean"
        );
        assert_eq!(cql_type_to_string("unknown"), "unknown");
    }
}
