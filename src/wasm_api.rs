#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_parse_to_json(grammar: &str, input: &str) -> Result<String, JsValue> {
    crate::parse_to_json(grammar, input).map_err(|e| JsValue::from_str(&e))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_parse_to_xml(grammar: &str, input: &str) -> Result<String, JsValue> {
    crate::parse_to_xml(grammar, input).map_err(|e| JsValue::from_str(&e))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_parse_to_yaml(grammar: &str, input: &str) -> Result<String, JsValue> {
    crate::parse_to_yaml(grammar, input).map_err(|e| JsValue::from_str(&e))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_parse_to_sql(grammar: &str, input: &str) -> Result<String, JsValue> {
    crate::parse_to_sql(grammar, input).map_err(|e| JsValue::from_str(&e))
}
