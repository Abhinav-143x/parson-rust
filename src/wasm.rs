#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::{parse_string, Value};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn parse_and_format(input: &str) -> String {
    match parse_string(input) {
        Ok(value) => {
            let json_str = crate::serializer::serialize_value_pretty(&value).unwrap_or_else(|_| "Serialization error".to_string());
            json_str
        }
        Err(e) => format!("Error parsing JSON: {:?}", e),
    }
}
