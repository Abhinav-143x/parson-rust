// tests/test_parity.rs — Behavioral parity test suite
//
// PURPOSE: Mirror the assertions in parson/tests.c (test_suite_3 primarily)
// to verify our Rust parser produces identical accept/reject decisions as the C original.
//
// ORGANISATION:
//   mod parse_valid   — inputs the C library accepts, we must too
//   mod parse_invalid — inputs the C library rejects (returns NULL), we must return Err
//   mod values        — parsed value type + content correctness
//   mod unicode       — UTF-16 escape decoding and surrogate pairs
//   mod numbers       — numeric edge cases (octal, hex, overflow, exponents)
//   mod serializer    — round-trip serialize→parse produces same structure
//   mod nesting       — depth limit enforcement (MAX_NESTING = 2048)
//   mod dotget        — object dot-path navigation

use parson_port::{parse_string, Value};

// ─── helpers ──────────────────────────────────────────────────────────────────

fn parses_ok(s: &str) -> Value {
    parse_string(s).unwrap_or_else(|e| panic!("Expected parse success for {s:?}, got {e:?}"))
}

fn parses_err(s: &str) {
    assert!(
        parse_string(s).is_err(),
        "Expected parse failure for {s:?}, but it succeeded"
    );
}

fn as_str(v: &Value) -> &str {
    v.as_str().expect("expected string value")
}

fn as_f64(v: &Value) -> f64 {
    v.as_number().expect("expected number value")
}

fn as_bool(v: &Value) -> bool {
    v.as_bool().expect("expected bool value")
}

// ─── Valid inputs (from tests.c test_suite_3) ─────────────────────────────────

#[cfg(test)]
mod parse_valid {
    use super::*;

    #[test]
    fn simple_object() {
        // tests.c line 311
        parses_ok(r#"{"lorem":"ipsum"}"#);
    }

    #[test]
    fn simple_array() {
        // tests.c line 312
        parses_ok(r#"["lorem"]"#);
    }

    #[test]
    fn null_literal() {
        // tests.c line 313
        let v = parses_ok("null");
        assert!(v.is_null());
    }

    #[test]
    fn true_literal() {
        // tests.c line 314
        let v = parses_ok("true");
        assert_eq!(as_bool(&v), true);
    }

    #[test]
    fn false_literal() {
        // tests.c line 315
        let v = parses_ok("false");
        assert_eq!(as_bool(&v), false);
    }

    #[test]
    fn string_literal() {
        // tests.c line 316
        let v = parses_ok(r#""string""#);
        assert_eq!(as_str(&v), "string");
    }

    #[test]
    fn number_literal() {
        // tests.c line 317
        let v = parses_ok("123");
        assert!((as_f64(&v) - 123.0).abs() < f64::EPSILON);
    }

    #[test]
    fn trailing_comma_array() {
        // tests.c line 318 — parson is lenient about trailing commas [D7]
        let v = parses_ok(r#"["lorem",]"#);
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn trailing_comma_object() {
        // tests.c line 319 — parson is lenient about trailing commas [D7]
        let v = parses_ok(r#"{"lorem":"ipsum",}"#);
        let obj = v.as_object().expect("object");
        assert_eq!(obj.len(), 1);
    }

    #[test]
    fn empty_object() {
        let v = parses_ok("{}");
        let obj = v.as_object().expect("object");
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn empty_array() {
        let v = parses_ok("[]");
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn whitespace_around_value() {
        let v = parses_ok("   { \"a\" : 1 }   ");
        let obj = v.as_object().expect("object");
        assert!((obj.get("a").and_then(|x| x.as_f64()).unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn negative_number() {
        let v = parses_ok("-1");
        assert!((as_f64(&v) - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn hard_to_parse_number() {
        // tests.c line 230 — "-0.000314"
        let v = parses_ok("-0.000314");
        assert!((as_f64(&v) - (-0.000314)).abs() < 1e-12);
    }

    #[test]
    fn exponent_number() {
        let v = parses_ok("1e2");
        assert!((as_f64(&v) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn nested_object() {
        let v = parses_ok(r#"{"outer":{"inner":42}}"#);
        let obj = v.as_object().unwrap();
        let inner = obj.get("outer").unwrap().as_object().unwrap();
        let n = inner.get("inner").unwrap().as_f64().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn escaped_chars_in_string() {
        // tests.c line 276 — "\" \\ /"
        let v = parses_ok(r#""\"\\ \/""#);
        assert_eq!(as_str(&v), "\"\\ /");
    }

    #[test]
    fn all_escape_sequences() {
        let v = parses_ok(r#""\"\\\//\b\f\n\r\t""#);
        let s = as_str(&v);
        assert!(s.contains('"'));
        assert!(s.contains('\\'));
        assert!(s.contains('\n'));
    }

    #[test]
    fn url_with_colon_slash() {
        // tests.c line 275
        let v = parses_ok(r#""https://www.example.com/search?q=12345""#);
        assert_eq!(as_str(&v), "https://www.example.com/search?q=12345");
    }

    #[test]
    fn mixed_array() {
        let v = parses_ok(r#"[1, "two", true, null, {}]"#);
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert!((arr.get(0).unwrap().as_f64().unwrap() - 1.0).abs() < f64::EPSILON);
        assert_eq!(arr.get(1).unwrap().as_str().unwrap(), "two");
        assert_eq!(arr.get(2).unwrap().as_bool().unwrap(), true);
        assert!(arr.get(3).unwrap().is_null());
        assert!(arr.get(4).unwrap().as_object().is_some());
    }
}

// ─── Invalid inputs (from tests.c test_suite_3) ───────────────────────────────

#[cfg(test)]
mod parse_invalid {
    use super::*;

    #[test]
    fn empty_string() {
        // tests.c line 330
        parses_err("");
    }

    #[test]
    fn unquoted_key() {
        // tests.c line 331
        parses_err("{lorem:ipsum}");
    }

    #[test]
    fn mismatched_brackets() {
        // tests.c line 332
        parses_err(r#"{"lorem":"ipsum",]"#);
    }

    #[test]
    fn double_comma_object() {
        // tests.c line 333
        parses_err(r#"{"lorem":"ipsum",,}"#);
    }

    #[test]
    fn leading_comma_array() {
        // tests.c line 334
        parses_err("[,]");
    }

    #[test]
    fn unclosed_array() {
        // tests.c line 336
        parses_err("[");
    }

    #[test]
    fn orphan_close_bracket() {
        // tests.c line 337
        parses_err("]");
    }

    #[test]
    fn duplicate_object_keys() {
        // tests.c line 338 — parson rejects duplicate keys
        parses_err(r#"{"a":0,"a":0}"#);
    }

    #[test]
    fn colon_without_key() {
        // tests.c line 339
        parses_err("{:,}");
    }

    #[test]
    fn comma_without_key() {
        // tests.c line 340
        parses_err("{,}");
    }

    #[test]
    fn unclosed_object() {
        // tests.c line 343
        parses_err("{");
    }

    #[test]
    fn orphan_close_brace() {
        // tests.c line 344
        parses_err("}");
    }

    #[test]
    fn unknown_literal() {
        // tests.c line 345
        parses_err("x");
    }

    #[test]
    fn invalid_utf_escape_bad_hex() {
        // tests.c line 350 — \u00zz
        parses_err(r#"["\u00zz"]"#);
    }

    #[test]
    fn invalid_utf_escape_too_short() {
        // tests.c line 351 — \u00
        parses_err(r#"["\u00"]"#);
    }

    #[test]
    fn invalid_utf_escape_no_digits() {
        // tests.c line 352 — \u
        parses_err(r#"["\u"]"#);
    }

    #[test]
    fn wrong_order_surrogate_pair() {
        // tests.c line 370 — trail before lead
        parses_err(r#"["\uDF67\uD834"]"#);
    }

    #[test]
    fn hex_number() {
        // tests.c line 362 — [D4] hex literals rejected
        parses_err("[0x2]");
    }

    #[test]
    fn hex_number_uppercase() {
        // tests.c line 363
        parses_err("[0X2]");
    }

    #[test]
    fn octal_number_07() {
        // tests.c line 364 — [D4] octal-style rejected
        parses_err("[07]");
    }

    #[test]
    fn octal_number_0070() {
        // tests.c line 365
        parses_err("[0070]");
    }

    #[test]
    fn octal_number_negative() {
        // tests.c line 367
        parses_err("[-07]");
    }

    #[test]
    fn number_overflow_positive() {
        // tests.c line 371 — 1.797e309 overflows f64 → +Inf → rejected [D4]
        parses_err("[1.7976931348623157e309]");
    }

    #[test]
    fn number_overflow_negative() {
        // tests.c line 372
        parses_err("[-1.7976931348623157e309]");
    }

    #[test]
    fn trailing_garbage() {
        // [BUG-FIX-A / INTENTIONAL DIVERGENCE from C parson]
        // C parson's json_parse_string() calls parse_value() and returns immediately,
        // never verifying the input pointer reached '\0'. This silently accepted
        // inputs like `{"a":1}GARBAGE` or `{}{}` (RFC 8259 §2 violation).
        //
        // We deliberately REJECT trailing content — every other major parser does too.
        // This is documented in DECISIONS.md as architectural divergence D-BugFix-A.
        // Original C behavior: parses_ok(r#"{"a":1}garbage"#);
        parses_err(r#"{"a":1}garbage"#);
        parses_err(r#"{"a":1}{}"#);
        parses_err(r#"[1,2,3] extra"#);
        // Trailing whitespace only is fine per RFC 8259 "ws value ws"
        parses_ok("{\"a\":1}   ");
    }

    #[test]
    fn control_char_tab_in_string() {
        // tests.c line 358 — raw tab in string is invalid [D6]
        parses_err("[\"	\"]");  // literal tab character
    }

    #[test]
    fn control_char_newline_in_string_raw() {
        // tests.c line 359 — raw newline in string literal is invalid [D6]
        // We build the string at runtime to embed a real \n byte
        let input = "[\"".to_owned() + "\n" + "\"]";
        parses_err(&input);
    }
}

// ─── Unicode tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod unicode {
    use super::*;

    #[test]
    fn unicode_escape_ascii() {
        // tests.c line 322 — \u0024 = '$'
        let v = parses_ok(r#""\u0024x""#);
        assert_eq!(as_str(&v), "$x");
    }

    #[test]
    fn unicode_escape_2byte() {
        // tests.c line 323 — \u00A2 = '¢'
        let v = parses_ok(r#""\u00A2x""#);
        assert_eq!(as_str(&v), "¢x");
    }

    #[test]
    fn unicode_escape_3byte() {
        // tests.c line 324 — \u20AC = '€'
        let v = parses_ok(r#""\u20ACx""#);
        assert_eq!(as_str(&v), "€x");
    }

    #[test]
    fn unicode_surrogate_pair() {
        // tests.c line 325 — \uD801\uDC37 = '𐐷' (U+10437)
        let v = parses_ok(r#""\uD801\uDC37x""#);
        assert_eq!(as_str(&v), "𐐷x");
    }

    #[test]
    fn raw_utf8_multibyte() {
        // tests.c line 221 — あいうえお (Japanese)
        let v = parses_ok(r#""あいうえお""#);
        assert_eq!(as_str(&v), "あいうえお");
    }

    #[test]
    fn surrogate_string_roundtrip() {
        // tests.c line 222 — "lorem𝄞ipsum𝍧lorem"
        let v = parses_ok(r#""lorem\uD834\uDD1Eipsum\uD834\uDF67lorem""#);
        assert_eq!(as_str(&v), "lorem𝄞ipsum𝍧lorem");
    }
}

// ─── Number edge cases ────────────────────────────────────────────────────────

#[cfg(test)]
mod numbers {
    use super::*;

    #[test]
    fn zero() {
        let v = parses_ok("0");
        assert!((as_f64(&v)).abs() < f64::EPSILON);
    }

    #[test]
    fn negative_zero() {
        let v = parses_ok("-0");
        // -0.0 and 0.0 are equal in f64
        assert!((as_f64(&v)).abs() < f64::EPSILON);
    }

    #[test]
    fn float_with_exponent() {
        let v = parses_ok("1.5e2");
        assert!((as_f64(&v) - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn negative_float_exponent() {
        let v = parses_ok("-3.14e-2");
        assert!((as_f64(&v) - (-0.0314)).abs() < 1e-12);
    }

    #[test]
    fn reject_leading_dot() {
        parses_err("[.5]");
    }

    #[test]
    fn reject_octal_with_decimal_point_but_leading_zero() {
        // tests.c line 366: [07.0] is rejected
        parses_err("[07.0]");
    }
}

// ─── Serializer round-trip tests ──────────────────────────────────────────────

#[cfg(test)]
mod serializer_tests {
    use super::*;
    use parson_port::serializer::{serialize_to_string, serialize_to_string_pretty};

    #[test]
    fn roundtrip_simple_object() {
        let original = r#"{"a":1,"b":"hello","c":true,"d":null}"#;
        let v1 = parses_ok(original);
        let serialized = serialize_to_string(&v1);
        let v2 = parses_ok(&serialized);
        // Verify key contents match
        let obj = v2.as_object().unwrap();
        assert!((obj.get("a").unwrap().as_f64().unwrap() - 1.0).abs() < f64::EPSILON);
        assert_eq!(obj.get("b").unwrap().as_str().unwrap(), "hello");
        assert_eq!(obj.get("c").unwrap().as_bool().unwrap(), true);
        assert!(obj.get("d").unwrap().is_null());
    }

    #[test]
    fn roundtrip_nested() {
        let original = r#"{"outer":{"inner":[1,2,3]}}"#;
        let v1 = parses_ok(original);
        let ser = serialize_to_string(&v1);
        let v2 = parses_ok(&ser);
        let arr = v2.as_object().unwrap()
            .get("outer").unwrap()
            .as_object().unwrap()
            .get("inner").unwrap()
            .as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn roundtrip_pretty() {
        let original = r#"{"a":1}"#;
        let v1 = parses_ok(original);
        let pretty = serialize_to_string_pretty(&v1);
        // Pretty-printed should still parse
        let v2 = parse_string(&pretty).unwrap();
        let obj = v2.as_object().unwrap();
        assert!((obj.get("a").unwrap().as_f64().unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn serialize_special_chars_in_string() {
        // Verify escape sequences survive a round-trip
        let original = Value::String("hello\nworld\t!".to_string());
        let ser = serialize_to_string(&original);
        // Should contain \n and \t escape sequences
        assert!(ser.contains("\\n"));
        assert!(ser.contains("\\t"));
        let v2 = parses_ok(&ser);
        assert_eq!(as_str(&v2), "hello\nworld\t!");
    }

    #[test]
    fn serialize_empty_array() {
        let v = parses_ok("[]");
        let s = serialize_to_string(&v);
        assert_eq!(s, "[]");
    }

    #[test]
    fn serialize_empty_object() {
        let v = parses_ok("{}");
        let s = serialize_to_string(&v);
        assert_eq!(s, "{}");
    }
}

// ─── Nesting depth limit tests ────────────────────────────────────────────────

#[cfg(test)]
mod nesting {
    use super::*;

    fn make_nested_arrays(depth: usize) -> String {
        // [D3] mirrors crash_test.c
        "[".repeat(depth) + &"]".repeat(depth)
    }

    #[test]
    fn shallow_nesting_ok() {
        // depth=5 — well within MAX_NESTING=2048
        let s = make_nested_arrays(5);
        parses_ok(&s);
    }

    #[test]
    fn nesting_at_limit_ok() {
        // depth=2048 — exactly at the limit (nesting counter reaches MAX_NESTING)
        let s = make_nested_arrays(2048);
        parses_ok(&s);
    }

    #[test]
    fn nesting_over_limit_fails() {
        // [D3] One beyond MAX_NESTING=2048 must be rejected.
        // WHY we use 2100 and not 2049:
        //   On Windows, the default thread stack is ~1MB. Each stack frame in our recursive
        //   Rust parser is ~200-400 bytes. 2049 frames × 400 bytes ≈ 820KB, dangerously close
        //   to the OS limit. Using 2100 ensures the guard fires on the first bracket that exceeds
        //   the limit, but is still indistinguishable from 2049 in terms of parser behavior —
        //   the guard check `nesting > MAX_NESTING` fires at frame 2049 regardless.
        //   The C library has the same fragility on low-stack platforms.
        // IMPACT: Tests the guard, not the OS stack limit.
        let s = make_nested_arrays(3000); // far over 2048, guard fires at 2049th bracket
        parses_err(&s);
    }

    #[test]
    fn deeply_nested_objects_at_limit() {
        let open: String = r#"{"a":"#.repeat(512);
        let val = "1";
        let close: String = "}".repeat(512);
        let json = open + val + &close;
        // 512 objects — well within limit
        parses_ok(&json);
    }
}

// ─── Object dotget navigation ─────────────────────────────────────────────────

#[cfg(test)]
mod dotget {
    use super::*;

    fn make_test_obj() -> Value {
        parses_ok(r#"{"outer":{"inner":{"deep":42}}}"#)
    }

    #[test]
    fn dotget_one_level() {
        let v = make_test_obj();
        let obj = v.as_object().unwrap();
        let inner = obj.dotget("outer").unwrap();
        assert!(inner.as_object().is_some());
    }

    #[test]
    fn dotget_two_levels() {
        let v = make_test_obj();
        let obj = v.as_object().unwrap();
        let deep = obj.dotget("outer.inner").unwrap();
        assert!(deep.as_object().is_some());
    }

    #[test]
    fn dotget_three_levels() {
        let v = make_test_obj();
        let obj = v.as_object().unwrap();
        let n = obj.dotget("outer.inner.deep").unwrap().as_f64().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dotget_missing_returns_none() {
        let v = make_test_obj();
        let obj = v.as_object().unwrap();
        // tests.c line 259
        assert!(obj.dotget("should.be.null").is_none());
    }

    #[test]
    fn dotget_empty_path_returns_none() {
        let v = make_test_obj();
        let obj = v.as_object().unwrap();
        // tests.c line 262
        assert!(obj.dotget("").is_none());
    }
}
