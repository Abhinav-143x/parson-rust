// Parser stub — full implementation comes in next commit.
use crate::value::ParsonError;
use crate::Value;

pub fn parse_string(_s: &str) -> Result<Value, ParsonError> {
    Err(ParsonError::Parse) // placeholder
}
