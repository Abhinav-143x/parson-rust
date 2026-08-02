/// Core value type for the JSON tree.
///
/// Mirrors `json_value_type` from parson.h but expressed as an idiomatic
/// Rust enum so the compiler can enforce exhaustive matching.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(crate::array::Array),
    Object(crate::object::Object),
}

/// Error type returned by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsonError {
    /// Input is not valid JSON.
    Parse,
    /// A requested key or index does not exist.
    NotFound,
    /// The value exists but is the wrong type.
    TypeError,
    /// An operation (e.g. set on a frozen object) is not allowed.
    InvalidOperation,
}

impl std::fmt::Display for ParsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsonError::Parse => write!(f, "JSON parse error"),
            ParsonError::NotFound => write!(f, "key or index not found"),
            ParsonError::TypeError => write!(f, "type mismatch"),
            ParsonError::InvalidOperation => write!(f, "invalid operation"),
        }
    }
}

impl std::error::Error for ParsonError {}

impl Value {
    /// Returns `true` if this value is JSON null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns the boolean payload, or `None`.
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Returns the numeric payload, or `None`.
    pub fn as_number(&self) -> Option<f64> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    /// Returns a reference to the string payload, or `None`.
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    /// Returns a reference to the array payload, or `None`.
    pub fn as_array(&self) -> Option<&crate::array::Array> {
        if let Value::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }

    /// Returns a reference to the object payload, or `None`.
    pub fn as_object(&self) -> Option<&crate::object::Object> {
        if let Value::Object(o) = self {
            Some(o)
        } else {
            None
        }
    }
}
