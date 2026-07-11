// PostgreSQL type conversion utilities

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::DataValue,
};
use chrono::{NaiveDate, NaiveDateTime};
use tokio_postgres::{types::Type, Row};

/// Converts PostgreSQL types to DataValue enum
pub struct PostgresTypeConverter;

impl PostgresTypeConverter {
    /// Convert a PostgreSQL row to a vector of DataValues
    pub fn row_to_data_values(&self, row: &Row) -> Vec<DataValue> {
        row.columns()
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                self.column_to_data_value(row, idx, col.type_())
                    .unwrap_or(DataValue::Null)
            })
            .collect()
    }

    /// Convert a single column value to DataValue
    pub fn column_to_data_value(
        &self,
        row: &Row,
        idx: usize,
        col_type: &Type,
    ) -> DriverResult<DataValue> {
        // Handle NULL values first
        if row
            .try_get::<_, Option<String>>(idx)
            .map(|opt| opt.is_none())
            .unwrap_or(false)
        {
            return Ok(DataValue::Null);
        }

        match *col_type {
            // Integer types
            Type::INT2 => {
                let val: i16 = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Integer(val as i64))
            }
            Type::INT4 => {
                let val: i32 = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Integer(val as i64))
            }
            Type::INT8 => {
                let val: i64 = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Integer(val))
            }

            // Float types
            Type::FLOAT4 => {
                let val: f32 = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Float(val as f64))
            }
            Type::FLOAT8 => {
                let val: f64 = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Float(val))
            }

            // Boolean
            Type::BOOL => {
                let val: bool = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Boolean(val))
            }

            // String types
            Type::VARCHAR | Type::TEXT | Type::BPCHAR | Type::NAME => {
                let val: String = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::String(val))
            }

            // Binary data
            Type::BYTEA => {
                let val: Vec<u8> = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Bytes(val))
            }

            // Date and time types
            Type::DATE => {
                // Get as string and parse
                let val: String = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                match NaiveDate::parse_from_str(&val, "%Y-%m-%d") {
                    Ok(date) => Ok(DataValue::from_date(date)),
                    Err(_) => Ok(DataValue::String(val)),
                }
            }
            Type::TIMESTAMP => {
                // Get as string and parse
                let val: String = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                match NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M:%S%.f") {
                    Ok(dt) => Ok(DataValue::from_datetime(dt)),
                    Err(_) => Ok(DataValue::String(val)),
                }
            }
            Type::TIMESTAMPTZ => {
                // Get as string
                let val: String = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::String(val))
            }

            // JSON types
            Type::JSON | Type::JSONB => {
                // Get as string and parse
                let val: String = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                match serde_json::from_str(&val) {
                    Ok(json) => Ok(DataValue::Json(json)),
                    Err(_) => Ok(DataValue::String(val)),
                }
            }

            // UUID
            Type::UUID => {
                // Get as string
                let val: String = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::String(val))
            }

            // Numeric/Decimal - convert to string for precision
            Type::NUMERIC => {
                // Try to get as f64 first, fall back to string
                if let Ok(val) = row.try_get::<_, f64>(idx) {
                    Ok(DataValue::Float(val))
                } else {
                    let val: String = row
                        .try_get(idx)
                        .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                    Ok(DataValue::String(val))
                }
            }

            // Arrays - handle common array types
            Type::INT4_ARRAY => {
                let val: Vec<i32> = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Array(
                    val.into_iter()
                        .map(|v| DataValue::Integer(v as i64))
                        .collect(),
                ))
            }
            Type::INT8_ARRAY => {
                let val: Vec<i64> = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Array(
                    val.into_iter().map(DataValue::Integer).collect(),
                ))
            }
            Type::TEXT_ARRAY | Type::VARCHAR_ARRAY => {
                let val: Vec<String> = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Array(
                    val.into_iter().map(DataValue::String).collect(),
                ))
            }
            Type::BOOL_ARRAY => {
                let val: Vec<bool> = row
                    .try_get(idx)
                    .map_err(|e| DriverError::TypeConversionError(e.to_string()))?;
                Ok(DataValue::Array(
                    val.into_iter().map(DataValue::Boolean).collect(),
                ))
            }

            // Fallback: try to convert to string
            _ => match row.try_get::<_, String>(idx) {
                Ok(val) => Ok(DataValue::String(val)),
                Err(e) => Err(DriverError::TypeConversionError(format!(
                    "Unsupported type {:?}: {}",
                    col_type, e
                ))),
            },
        }
    }

    /// Map PostgreSQL type name to a standard string representation
    pub fn type_name_to_string(type_name: &str) -> String {
        let result = match type_name.to_uppercase().as_str() {
            "INT2" | "SMALLINT" => "SMALLINT",
            "INT4" | "INTEGER" | "INT" => "INTEGER",
            "INT8" | "BIGINT" => "BIGINT",
            "FLOAT4" | "REAL" => "REAL",
            "FLOAT8" | "DOUBLE PRECISION" => "DOUBLE PRECISION",
            "NUMERIC" | "DECIMAL" => "NUMERIC",
            "VARCHAR" | "CHARACTER VARYING" => "VARCHAR",
            "CHAR" | "CHARACTER" => "CHAR",
            "TEXT" => "TEXT",
            "BOOL" | "BOOLEAN" => "BOOLEAN",
            "DATE" => "DATE",
            "TIMESTAMP" | "TIMESTAMP WITHOUT TIME ZONE" => "TIMESTAMP",
            "TIMESTAMPTZ" | "TIMESTAMP WITH TIME ZONE" => "TIMESTAMPTZ",
            "TIME" | "TIME WITHOUT TIME ZONE" => "TIME",
            "TIMETZ" | "TIME WITH TIME ZONE" => "TIMETZ",
            "BYTEA" => "BYTEA",
            "JSON" => "JSON",
            "JSONB" => "JSONB",
            "UUID" => "UUID",
            "INET" => "INET",
            "CIDR" => "CIDR",
            "MACADDR" => "MACADDR",
            other => return other.to_string(),
        };
        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_name_normalization() {
        assert_eq!(
            PostgresTypeConverter::type_name_to_string("int4"),
            "INTEGER"
        );
        assert_eq!(PostgresTypeConverter::type_name_to_string("int8"), "BIGINT");
        assert_eq!(
            PostgresTypeConverter::type_name_to_string("varchar"),
            "VARCHAR"
        );
        assert_eq!(PostgresTypeConverter::type_name_to_string("text"), "TEXT");
        assert_eq!(
            PostgresTypeConverter::type_name_to_string("bool"),
            "BOOLEAN"
        );
        assert_eq!(
            PostgresTypeConverter::type_name_to_string("timestamptz"),
            "TIMESTAMPTZ"
        );
        assert_eq!(PostgresTypeConverter::type_name_to_string("json"), "JSON");
        assert_eq!(PostgresTypeConverter::type_name_to_string("jsonb"), "JSONB");
    }

    #[test]
    fn test_data_value_conversions() {
        // Test integer conversion
        let int_val = DataValue::from_i32(42);
        assert_eq!(int_val, DataValue::Integer(42));

        // Test float conversion
        let float_val = DataValue::from_f64(3.14);
        assert_eq!(float_val, DataValue::Float(3.14));

        // Test boolean conversion
        let bool_val = DataValue::from_bool(true);
        assert_eq!(bool_val, DataValue::Boolean(true));

        // Test string conversion
        let str_val = DataValue::from_string("test".to_string());
        assert_eq!(str_val, DataValue::String("test".to_string()));

        // Test null check
        assert!(DataValue::Null.is_null());
        assert!(!DataValue::Integer(0).is_null());
    }

    #[test]
    fn test_data_value_display() {
        assert_eq!(DataValue::Null.to_string(), "NULL");
        assert_eq!(DataValue::Integer(42).to_string(), "42");
        assert_eq!(DataValue::Float(3.14).to_string(), "3.14");
        assert_eq!(DataValue::Boolean(true).to_string(), "true");
        assert_eq!(DataValue::String("test".to_string()).to_string(), "test");

        let arr = DataValue::Array(vec![
            DataValue::Integer(1),
            DataValue::Integer(2),
            DataValue::Integer(3),
        ]);
        assert_eq!(arr.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_json_data_value() {
        let json_val = serde_json::json!({
            "key": "value",
            "number": 42
        });
        let data_val = DataValue::Json(json_val);

        match data_val {
            DataValue::Json(val) => {
                assert_eq!(val["key"], "value");
                assert_eq!(val["number"], 42);
            }
            _ => panic!("Expected Json variant"),
        }
    }

    #[test]
    fn test_date_conversion() {
        use chrono::NaiveDate;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let data_val = DataValue::from_date(date);

        match data_val {
            DataValue::Date(s) => assert_eq!(s, "2024-01-15"),
            _ => panic!("Expected Date variant"),
        }
    }

    #[test]
    fn test_datetime_conversion() {
        use chrono::NaiveDate;
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(14, 30, 45)
            .unwrap();
        let data_val = DataValue::from_datetime(dt);

        match data_val {
            DataValue::DateTime(s) => assert_eq!(s, "2024-01-15T14:30:45"),
            _ => panic!("Expected DateTime variant"),
        }
    }

    #[test]
    fn test_bytes_conversion() {
        let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
        let data_val = DataValue::from_bytes(bytes.clone());

        match data_val {
            DataValue::Bytes(b) => assert_eq!(b, bytes),
            _ => panic!("Expected Bytes variant"),
        }
    }
}
