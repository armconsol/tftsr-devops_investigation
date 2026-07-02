// Redis type conversion utilities

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::DataValue,
};
use redis::Value as RedisValue;

/// Convert Redis value to DataValue
pub fn redis_value_to_data_value(value: &RedisValue) -> DriverResult<DataValue> {
    match value {
        RedisValue::Nil => Ok(DataValue::Null),

        RedisValue::Int(i) => Ok(DataValue::Integer(*i)),

        RedisValue::Data(bytes) => {
            // Try to convert bytes to UTF-8 string, fallback to bytes if invalid
            match String::from_utf8(bytes.clone()) {
                Ok(s) => Ok(DataValue::String(s)),
                Err(_) => Ok(DataValue::Bytes(bytes.clone())),
            }
        }

        RedisValue::Bulk(values) => {
            let converted: Result<Vec<DataValue>, DriverError> =
                values.iter().map(redis_value_to_data_value).collect();
            Ok(DataValue::Array(converted?))
        }

        RedisValue::Status(s) => Ok(DataValue::String(s.clone())),

        RedisValue::Okay => Ok(DataValue::String("OK".to_string())),
    }
}

/// Parse Redis command string into command parts
/// Handles both simple commands and quoted arguments
pub fn parse_redis_command(command: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars = command.chars();

    for c in chars {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_value_conversion_nil() {
        let value = RedisValue::Nil;
        let result = redis_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_redis_value_conversion_int() {
        let value = RedisValue::Int(42);
        let result = redis_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::Integer(42));
    }

    #[test]
    fn test_redis_value_conversion_string() {
        let value = RedisValue::Data(b"hello".to_vec());
        let result = redis_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::String("hello".to_string()));
    }

    #[test]
    fn test_redis_value_conversion_status() {
        let value = RedisValue::Status("OK".to_string());
        let result = redis_value_to_data_value(&value).unwrap();
        assert_eq!(result, DataValue::String("OK".to_string()));
    }

    #[test]
    fn test_redis_value_conversion_bulk() {
        let value = RedisValue::Bulk(vec![
            RedisValue::Data(b"key1".to_vec()),
            RedisValue::Data(b"value1".to_vec()),
        ]);
        let result = redis_value_to_data_value(&value).unwrap();

        match result {
            DataValue::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], DataValue::String("key1".to_string()));
                assert_eq!(arr[1], DataValue::String("value1".to_string()));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_redis_command_simple() {
        let cmd = "GET mykey";
        let parts = parse_redis_command(cmd);
        assert_eq!(parts, vec!["GET", "mykey"]);
    }

    #[test]
    fn test_parse_redis_command_multiple_args() {
        let cmd = "SET mykey myvalue";
        let parts = parse_redis_command(cmd);
        assert_eq!(parts, vec!["SET", "mykey", "myvalue"]);
    }

    #[test]
    fn test_parse_redis_command_with_quotes() {
        let cmd = r#"SET mykey "hello world""#;
        let parts = parse_redis_command(cmd);
        assert_eq!(parts, vec!["SET", "mykey", "hello world"]);
    }

    #[test]
    fn test_parse_redis_command_extra_spaces() {
        let cmd = "GET  mykey   ";
        let parts = parse_redis_command(cmd);
        assert_eq!(parts, vec!["GET", "mykey"]);
    }

    #[test]
    fn test_parse_redis_command_lrange() {
        let cmd = "LRANGE mylist 0 -1";
        let parts = parse_redis_command(cmd);
        assert_eq!(parts, vec!["LRANGE", "mylist", "0", "-1"]);
    }
}
