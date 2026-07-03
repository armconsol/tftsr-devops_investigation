// MySQL type conversion utilities

use crate::db_drivers::{error::DriverResult, types::DataValue};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use mysql_async::{consts::ColumnType, Row, Value};

/// Converts MySQL types to DataValue enum
pub struct MySQLTypeConverter;

impl MySQLTypeConverter {
    /// Convert a MySQL row to a vector of DataValues
    pub fn row_to_data_values(&self, row: &Row) -> Vec<DataValue> {
        row.columns_ref()
            .iter()
            .enumerate()
            .map(|(idx, _col)| {
                self.column_to_data_value(row, idx)
                    .unwrap_or(DataValue::Null)
            })
            .collect()
    }

    /// Convert a single column value to DataValue
    pub fn column_to_data_value(&self, row: &Row, idx: usize) -> DriverResult<DataValue> {
        let value: Value = row.get(idx).unwrap_or(Value::NULL);

        match value {
            Value::NULL => Ok(DataValue::Null),

            // Integer types
            Value::Bytes(bytes) => {
                // MySQL returns most values as bytes, need to parse
                let s = String::from_utf8_lossy(&bytes);
                // Try to parse as different types
                if let Ok(i) = s.parse::<i64>() {
                    Ok(DataValue::Integer(i))
                } else if let Ok(f) = s.parse::<f64>() {
                    Ok(DataValue::Float(f))
                } else {
                    Ok(DataValue::String(s.to_string()))
                }
            }

            Value::Int(i) => Ok(DataValue::Integer(i)),

            Value::UInt(u) => Ok(DataValue::Integer(u as i64)),

            // Float types
            Value::Float(f) => Ok(DataValue::Float(f as f64)),
            Value::Double(d) => Ok(DataValue::Float(d)),

            // Date and time types
            Value::Date(year, month, day, hour, minute, second, _micro) => {
                if hour == 0 && minute == 0 && second == 0 {
                    // Pure date
                    if let Some(date) =
                        NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                    {
                        Ok(DataValue::from_date(date))
                    } else {
                        Ok(DataValue::Null)
                    }
                } else {
                    // DateTime
                    if let Some(date) =
                        NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                    {
                        if let Some(time) =
                            NaiveTime::from_hms_opt(hour as u32, minute as u32, second as u32)
                        {
                            let dt = NaiveDateTime::new(date, time);
                            Ok(DataValue::from_datetime(dt))
                        } else {
                            Ok(DataValue::Null)
                        }
                    } else {
                        Ok(DataValue::Null)
                    }
                }
            }

            Value::Time(is_negative, days, hours, minutes, seconds, _micro) => {
                // Convert time to string representation
                let sign = if is_negative { "-" } else { "" };
                let total_hours = (days * 24) + hours as u32;
                let time_str = format!("{}{}:{:02}:{:02}", sign, total_hours, minutes, seconds);
                Ok(DataValue::String(time_str))
            }
        }
    }

    /// Map MySQL type name to a standard string representation
    pub fn type_name_to_string(type_name: &str) -> String {
        // Extract base type from full type definition (e.g., "varchar(255)" -> "VARCHAR")
        let base_type = type_name
            .split('(')
            .next()
            .unwrap_or(type_name)
            .trim()
            .to_uppercase();

        match base_type.as_str() {
            "TINYINT" => "TINYINT",
            "SMALLINT" => "SMALLINT",
            "MEDIUMINT" => "MEDIUMINT",
            "INT" | "INTEGER" => "INTEGER",
            "BIGINT" => "BIGINT",
            "FLOAT" => "FLOAT",
            "DOUBLE" | "DOUBLE PRECISION" => "DOUBLE",
            "DECIMAL" | "NUMERIC" => "DECIMAL",
            "CHAR" => "CHAR",
            "VARCHAR" => "VARCHAR",
            "TINYTEXT" => "TINYTEXT",
            "TEXT" => "TEXT",
            "MEDIUMTEXT" => "MEDIUMTEXT",
            "LONGTEXT" => "LONGTEXT",
            "BINARY" => "BINARY",
            "VARBINARY" => "VARBINARY",
            "TINYBLOB" => "TINYBLOB",
            "BLOB" => "BLOB",
            "MEDIUMBLOB" => "MEDIUMBLOB",
            "LONGBLOB" => "LONGBLOB",
            "DATE" => "DATE",
            "TIME" => "TIME",
            "DATETIME" => "DATETIME",
            "TIMESTAMP" => "TIMESTAMP",
            "YEAR" => "YEAR",
            "ENUM" => "ENUM",
            "SET" => "SET",
            "JSON" => "JSON",
            "BIT" => "BIT",
            "BOOLEAN" | "BOOL" => "BOOLEAN",
            "GEOMETRY" => "GEOMETRY",
            "POINT" => "POINT",
            "LINESTRING" => "LINESTRING",
            "POLYGON" => "POLYGON",
            other => other,
        }
        .to_string()
    }

    /// Get MySQL column type from ColumnType enum
    pub fn column_type_to_string(col_type: ColumnType) -> String {
        match col_type {
            ColumnType::MYSQL_TYPE_DECIMAL => "DECIMAL",
            ColumnType::MYSQL_TYPE_TINY => "TINYINT",
            ColumnType::MYSQL_TYPE_SHORT => "SMALLINT",
            ColumnType::MYSQL_TYPE_LONG => "INTEGER",
            ColumnType::MYSQL_TYPE_FLOAT => "FLOAT",
            ColumnType::MYSQL_TYPE_DOUBLE => "DOUBLE",
            ColumnType::MYSQL_TYPE_NULL => "NULL",
            ColumnType::MYSQL_TYPE_TIMESTAMP => "TIMESTAMP",
            ColumnType::MYSQL_TYPE_LONGLONG => "BIGINT",
            ColumnType::MYSQL_TYPE_INT24 => "MEDIUMINT",
            ColumnType::MYSQL_TYPE_DATE => "DATE",
            ColumnType::MYSQL_TYPE_TIME => "TIME",
            ColumnType::MYSQL_TYPE_DATETIME => "DATETIME",
            ColumnType::MYSQL_TYPE_YEAR => "YEAR",
            ColumnType::MYSQL_TYPE_NEWDATE => "DATE",
            ColumnType::MYSQL_TYPE_VARCHAR => "VARCHAR",
            ColumnType::MYSQL_TYPE_BIT => "BIT",
            ColumnType::MYSQL_TYPE_TIMESTAMP2 => "TIMESTAMP",
            ColumnType::MYSQL_TYPE_DATETIME2 => "DATETIME",
            ColumnType::MYSQL_TYPE_TIME2 => "TIME",
            ColumnType::MYSQL_TYPE_JSON => "JSON",
            ColumnType::MYSQL_TYPE_NEWDECIMAL => "DECIMAL",
            ColumnType::MYSQL_TYPE_ENUM => "ENUM",
            ColumnType::MYSQL_TYPE_SET => "SET",
            ColumnType::MYSQL_TYPE_TINY_BLOB => "TINYBLOB",
            ColumnType::MYSQL_TYPE_MEDIUM_BLOB => "MEDIUMBLOB",
            ColumnType::MYSQL_TYPE_LONG_BLOB => "LONGBLOB",
            ColumnType::MYSQL_TYPE_BLOB => "BLOB",
            ColumnType::MYSQL_TYPE_VAR_STRING => "VARCHAR",
            ColumnType::MYSQL_TYPE_STRING => "CHAR",
            ColumnType::MYSQL_TYPE_GEOMETRY => "GEOMETRY",
            _ => "UNKNOWN",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_name_normalization() {
        assert_eq!(MySQLTypeConverter::type_name_to_string("int"), "INTEGER");
        assert_eq!(MySQLTypeConverter::type_name_to_string("INT"), "INTEGER");
        assert_eq!(MySQLTypeConverter::type_name_to_string("bigint"), "BIGINT");
        assert_eq!(
            MySQLTypeConverter::type_name_to_string("varchar(255)"),
            "VARCHAR"
        );
        assert_eq!(
            MySQLTypeConverter::type_name_to_string("VARCHAR(100)"),
            "VARCHAR"
        );
        assert_eq!(MySQLTypeConverter::type_name_to_string("text"), "TEXT");
        assert_eq!(
            MySQLTypeConverter::type_name_to_string("datetime"),
            "DATETIME"
        );
        assert_eq!(
            MySQLTypeConverter::type_name_to_string("timestamp"),
            "TIMESTAMP"
        );
        assert_eq!(MySQLTypeConverter::type_name_to_string("json"), "JSON");
        assert_eq!(
            MySQLTypeConverter::type_name_to_string("decimal(10,2)"),
            "DECIMAL"
        );
    }

    #[test]
    fn test_column_type_to_string() {
        assert_eq!(
            MySQLTypeConverter::column_type_to_string(ColumnType::MYSQL_TYPE_LONG),
            "INTEGER"
        );
        assert_eq!(
            MySQLTypeConverter::column_type_to_string(ColumnType::MYSQL_TYPE_LONGLONG),
            "BIGINT"
        );
        assert_eq!(
            MySQLTypeConverter::column_type_to_string(ColumnType::MYSQL_TYPE_VARCHAR),
            "VARCHAR"
        );
        assert_eq!(
            MySQLTypeConverter::column_type_to_string(ColumnType::MYSQL_TYPE_BLOB),
            "BLOB"
        );
        assert_eq!(
            MySQLTypeConverter::column_type_to_string(ColumnType::MYSQL_TYPE_DATETIME),
            "DATETIME"
        );
        assert_eq!(
            MySQLTypeConverter::column_type_to_string(ColumnType::MYSQL_TYPE_JSON),
            "JSON"
        );
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
