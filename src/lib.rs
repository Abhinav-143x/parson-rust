#![forbid(unsafe_code)]
pub mod value;
pub mod array;
pub mod object;
pub mod parser;
pub mod serializer;
pub mod validate;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use value::{Value, ParsonError};
pub use array::Array;
pub use object::Object;
pub use serializer::{serialize_to_string, serialize_to_string_pretty};
pub use validate::validate;

/// Parse a JSON string. Returns the root `Value` or a `ParsonError`.
///
/// Equivalent to `json_parse_string()` in parson.h.
pub fn parse_string(s: &str) -> Result<Value, ParsonError> {
    parser::parse_string(s)
}

/// Parse a JSON file from disk.
pub fn parse_file(path: &str) -> Result<Value, ParsonError> {
    let content = std::fs::read_to_string(path).map_err(|_| ParsonError::Parse)?;
    parse_string(&content)
}

/// Parse a JSON string supporting line and block comments.
///
/// Equivalent to `json_parse_string_with_comments()` in parson.h.
pub fn parse_string_with_comments(s: &str) -> Result<Value, ParsonError> {
    parser::parse_string_with_comments(s)
}

/// Parse a JSON file from disk with comment support.
///
/// Equivalent to `json_parse_file_with_comments()` in parson.h.
pub fn parse_file_with_comments(path: &str) -> Result<Value, ParsonError> {
    let content = std::fs::read_to_string(path).map_err(|_| ParsonError::Parse)?;
    parse_string_with_comments(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_with_comments() {
        let input = "{\n  // Single line comment\n  \"key\": /* inline block comment */ \"value\"\n}";
        let parsed = parse_string_with_comments(input).unwrap();
        let obj = parsed.as_object().unwrap();
        assert_eq!(obj.get("key").and_then(|v| v.as_str()), Some("value"));
    }
}

