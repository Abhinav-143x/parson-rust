pub mod value;
pub mod array;
pub mod object;
pub mod parser;
pub mod serializer;

pub use value::{Value, ParsonError};
pub use array::Array;
pub use object::Object;
pub use serializer::{serialize_to_string, serialize_to_string_pretty};

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
