#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn parse_and_format(input: &str) -> String {
    match crate::parser::parse_string(input) {
        Ok(value) => {
            crate::serializer::serialize_to_string_pretty(&value)
        }
        Err(e) => format!("Error parsing JSON: {:?}", e),
    }
}
