use crate::Value;

pub fn serialize_to_string(value: &Value) -> String {
    let mut result = String::new();
    serialize_internal(value, &mut result, false, 0);
    result
}

pub fn serialize_to_string_pretty(value: &Value) -> String {
    let mut result = String::new();
    serialize_internal(value, &mut result, true, 0);
    result
}

fn serialize_internal(value: &Value, out: &mut String, pretty: bool, level: usize) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => if *b { out.push_str("true") } else { out.push_str("false") },
        Value::Number(n) => out.push_str(&format_number(*n)),
        Value::String(s) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '\"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '/' => out.push_str("\\/"),  /* match parson default: parson_escape_slashes = 1 */
                    '\x08' => out.push_str("\\b"),
                    '\x0C' => out.push_str("\\f"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c <= '\x1F' => {
                        use std::fmt::Write;
                        write!(out, "\\u{:04x}", c as u32).unwrap();
                    }
                    _ => out.push(c),
                }
            }
            out.push('"');
        }
        Value::Array(arr) => {
            out.push('[');
            let len = arr.len();
            for (i, item) in arr.iter().enumerate() {
                if pretty {
                    out.push('\n');
                    out.push_str(&"    ".repeat(level + 1));  /* PARSON_INDENT_STR = 4 spaces */
                }
                serialize_internal(item, out, pretty, level + 1);
                if i < len - 1 {
                    out.push(',');
                }
            }
            if pretty && len > 0 {
                out.push('\n');
                out.push_str(&"    ".repeat(level));
            }
            out.push(']');
        }
        Value::Object(obj) => {
            out.push('{');
            let len = obj.len();
            for (i, (k, v)) in obj.iter().enumerate() {
                if pretty {
                    out.push('\n');
                    out.push_str(&"    ".repeat(level + 1));  /* PARSON_INDENT_STR = 4 spaces */
                }
                serialize_internal(&Value::String(k.clone()), out, pretty, level + 1);
                if pretty {
                    out.push_str(": ");
                } else {
                    out.push(':');
                }
                serialize_internal(v, out, pretty, level + 1);
                if i < len - 1 {
                    out.push(',');
                }
            }
            if pretty && len > 0 {
                out.push('\n');
                out.push_str(&"    ".repeat(level));
            }
            out.push('}');
        }
    }
}

/// Format a float using C parson's default format: "%1.17g".
/// Matches parson.c line 68: `#define PARSON_DEFAULT_FLOAT_FORMAT "%1.17g"`
fn format_number(n: f64) -> String {
    // %g removes trailing zeros; %.17g gives up to 17 significant digits.
    // Rust's {:.*} with 17 sig-figs via {:.17e} isn't identical to %g,
    // so we implement %1.17g semantics: use up to 17 significant digits,
    // switch to scientific notation when exponent < -4 or >= precision.
    if n == 0.0 {
        return "0".to_string();
    }
    // Format with full precision then strip trailing zeros like %g.
    let full = format!("{:.17e}", n);
    // Parse mantissa and exponent from the scientific notation string.
    let parts: Vec<&str> = full.splitn(2, 'e').collect();
    if parts.len() != 2 { return n.to_string(); }
    let exp: i32 = parts[1].parse().unwrap_or(0);
    if exp >= -4 && exp < 17 {
        // Fixed notation: format with enough decimal places.
        let decimals = (16 - exp).max(0) as usize;
        let fixed = format!("{:.prec$}", n, prec = decimals);
        // Strip trailing zeros after decimal point.
        if fixed.contains('.') {
            let stripped = fixed.trim_end_matches('0').trim_end_matches('.');
            stripped.to_string()
        } else {
            fixed
        }
    } else {
        // Scientific notation: strip trailing zeros from mantissa.
        let mantissa = parts[0].trim_end_matches('0').trim_end_matches('.');
        format!("{}e{}", mantissa, exp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Array, Object};

    #[test]
    fn test_serialize_primitives() {
        assert_eq!(serialize_to_string(&Value::Null), "null");
        assert_eq!(serialize_to_string(&Value::Bool(true)), "true");
        assert_eq!(serialize_to_string(&Value::Number(42.5)), "42.5");
        assert_eq!(serialize_to_string(&Value::String("hello \"world\"\n".to_string())), "\"hello \\\"world\\\"\\n\"");
    }

    #[test]
    fn test_serialize_array_and_object() {
        let mut arr = Array::new();
        arr.push(Value::Number(1.0));
        arr.push(Value::Bool(false));
        assert_eq!(serialize_to_string(&Value::Array(arr)), "[1,false]");

        let mut obj = Object::new();
        obj.set("name".to_string(), Value::String("parson".to_string()));
        obj.set("valid".to_string(), Value::Bool(true));
        assert_eq!(serialize_to_string(&Value::Object(obj.clone())), "{\"name\":\"parson\",\"valid\":true}");

        let pretty = serialize_to_string_pretty(&Value::Object(obj));
        let expected = "{\n    \"name\": \"parson\",\n    \"valid\": true\n}";  /* 4-space indent matches PARSON_INDENT_STR */
        assert_eq!(pretty, expected);
    }
}
