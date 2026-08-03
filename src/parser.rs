use crate::array::Array;
use crate::object::Object;
use crate::value::ParsonError;
use crate::Value;
use std::str::Chars;
use std::iter::Peekable;

const MAX_NESTING_DEPTH: usize = 2048;

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    depth: usize,
    allow_comments: bool,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, allow_comments: bool) -> Self {
        Self {
            chars: input.chars().peekable(),
            depth: 0,
            allow_comments,
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), ParsonError> {
        loop {
            match self.chars.peek() {
                Some(&' ') | Some(&'\t') | Some(&'\r') | Some(&'\n') => {
                    self.chars.next();
                }
                Some(&'/') if self.allow_comments => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some(&'/') => {
                            self.chars.next();
                            while let Some(c) = self.chars.next() {
                                if c == '\n' {
                                    break;
                                }
                            }
                        }
                        Some(&'*') => {
                            self.chars.next();
                            let mut closed = false;
                            while let Some(c) = self.chars.next() {
                                if c == '*' && self.chars.peek() == Some(&'/') {
                                    self.chars.next();
                                    closed = true;
                                    break;
                                }
                            }
                            if !closed {
                                return Err(ParsonError::Parse);
                            }
                        }
                        _ => return Err(ParsonError::Parse),
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_value(&mut self) -> Result<Value, ParsonError> {
        self.skip_whitespace_and_comments()?;
        if self.depth >= MAX_NESTING_DEPTH {
            return Err(ParsonError::Parse);
        }

        match self.chars.peek() {
            Some(&'"') => self.parse_string_value().map(Value::String),
            Some(&'{') => self.parse_object().map(Value::Object),
            Some(&'[') => self.parse_array().map(Value::Array),
            Some(&'n') => self.parse_literal("null", Value::Null),
            Some(&'t') => self.parse_literal("true", Value::Bool(true)),
            Some(&'f') => self.parse_literal("false", Value::Bool(false)),
            Some(&'-') | Some(&('0'..='9')) => self.parse_number(),
            _ => Err(ParsonError::Parse),
        }
    }

    fn parse_literal(&mut self, expected: &str, value: Value) -> Result<Value, ParsonError> {
        for ch in expected.chars() {
            if self.chars.next() != Some(ch) {
                return Err(ParsonError::Parse);
            }
        }
        Ok(value)
    }

    fn parse_string_raw(&mut self) -> Result<String, ParsonError> {
        if self.chars.next() != Some('"') {
            return Err(ParsonError::Parse);
        }
        let mut res = String::new();
        loop {
            match self.chars.next() {
                None => return Err(ParsonError::Parse),
                Some('"') => break,
                Some('\\') => match self.chars.next() {
                    Some('"') => res.push('"'),
                    Some('\\') => res.push('\\'),
                    Some('/') => res.push('/'),
                    Some('b') => res.push('\x08'),
                    Some('f') => res.push('\x0C'),
                    Some('n') => res.push('\n'),
                    Some('r') => res.push('\r'),
                    Some('t') => res.push('\t'),
                    Some('u') => {
                        let u1 = self.parse_hex4()?;
                        if (0xD800..=0xDBFF).contains(&u1) {
                            if self.chars.next() != Some('\\') || self.chars.next() != Some('u') {
                                return Err(ParsonError::Parse);
                            }
                            let u2 = self.parse_hex4()?;
                            if !(0xDC00..=0xDFFF).contains(&u2) {
                                return Err(ParsonError::Parse);
                            }
                            let codepoint = 0x10000 + (((u1 as u32 - 0xD800) << 10) | (u2 as u32 - 0xDC00));
                            if let Some(ch) = char::from_u32(codepoint) {
                                res.push(ch);
                            } else {
                                return Err(ParsonError::Parse);
                            }
                        } else if let Some(ch) = char::from_u32(u1 as u32) {
                            res.push(ch);
                        } else {
                            return Err(ParsonError::Parse);
                        }
                    }
                    _ => return Err(ParsonError::Parse),
                },
                Some(c) if c < ' ' => return Err(ParsonError::Parse),
                Some(c) => res.push(c),
            }
        }
        Ok(res)
    }

    fn parse_hex4(&mut self) -> Result<u16, ParsonError> {
        let mut val = 0;
        for _ in 0..4 {
            let c = self.chars.next().ok_or(ParsonError::Parse)?;
            let digit = c.to_digit(16).ok_or(ParsonError::Parse)?;
            val = (val << 4) | (digit as u16);
        }
        Ok(val)
    }

    fn parse_string_value(&mut self) -> Result<String, ParsonError> {
        self.parse_string_raw()
    }

    fn parse_number(&mut self) -> Result<Value, ParsonError> {
        let mut s = String::new();
        if let Some(&'-') = self.chars.peek() {
            s.push(self.chars.next().unwrap());
        }

        // Track whether integer part is bare '0' — C's is_decimal() rejects
        // 0eN and -0eN (string[0]=='0' && string[1]!='.') but allows 0.XeN.
        let mut leading_zero = false;
        match self.chars.peek() {
            Some(&'0') => {
                s.push(self.chars.next().unwrap());
                leading_zero = true;
            }
            Some(&('1'..='9')) => {
                s.push(self.chars.next().unwrap());
                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_digit() {
                        s.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }
            }
            _ => return Err(ParsonError::Parse),
        }

        let mut had_dot = false;
        if let Some(&'.') = self.chars.peek() {
            had_dot = true;
            s.push(self.chars.next().unwrap());
            let mut has_digits = false;
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    s.push(self.chars.next().unwrap());
                    has_digits = true;
                } else {
                    break;
                }
            }
            if !has_digits {
                return Err(ParsonError::Parse);
            }
        }

        // C parson's is_decimal rejects 0eN and -0eN:
        //   if (length > 1 && string[0] == '0' && string[1] != '.') return FALSE;
        // So bare '0' or '-0' may only be followed by '.' — not 'e'/'E'.
        if leading_zero && !had_dot {
            if let Some(&'e') | Some(&'E') = self.chars.peek() {
                return Err(ParsonError::Parse);
            }
        }

        if let Some(&'e') | Some(&'E') = self.chars.peek() {
            s.push(self.chars.next().unwrap());
            if let Some(&'+') | Some(&'-') = self.chars.peek() {
                s.push(self.chars.next().unwrap());
            }
            let mut has_digits = false;
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    s.push(self.chars.next().unwrap());
                    has_digits = true;
                } else {
                    break;
                }
            }
            if !has_digits {
                return Err(ParsonError::Parse);
            }
        }

        match s.parse::<f64>() {
            Ok(num) if !num.is_infinite() => Ok(Value::Number(num)),
            _ => Err(ParsonError::Parse),
        }
    }

    fn parse_array(&mut self) -> Result<Array, ParsonError> {
        if self.chars.next() != Some('[') {
            return Err(ParsonError::Parse);
        }
        self.depth += 1;
        let mut arr = Array::new();
        self.skip_whitespace_and_comments()?;

        if self.chars.peek() == Some(&']') {
            self.chars.next();
            self.depth -= 1;
            return Ok(arr);
        }

        loop {
            let val = self.parse_value()?;
            arr.push(val);
            self.skip_whitespace_and_comments()?;
            match self.chars.next() {
                Some(',') => {
                    self.skip_whitespace_and_comments()?;
                    if self.chars.peek() == Some(&']') {
                        self.chars.next();
                        self.depth -= 1;
                        return Ok(arr);
                    }
                }
                Some(']') => {
                    self.depth -= 1;
                    return Ok(arr);
                }
                _ => return Err(ParsonError::Parse),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Object, ParsonError> {
        if self.chars.next() != Some('{') {
            return Err(ParsonError::Parse);
        }
        self.depth += 1;
        let mut obj = Object::new();
        self.skip_whitespace_and_comments()?;

        if self.chars.peek() == Some(&'}') {
            self.chars.next();
            self.depth -= 1;
            return Ok(obj);
        }

        loop {
            self.skip_whitespace_and_comments()?;
            if self.chars.peek() != Some(&'"') {
                return Err(ParsonError::Parse);
            }
            let key = self.parse_string_raw()?;
            self.skip_whitespace_and_comments()?;
            if self.chars.next() != Some(':') {
                return Err(ParsonError::Parse);
            }
            let val = self.parse_value()?;
            if obj.get(&key).is_some() {
                return Err(ParsonError::Parse);
            }
            obj.set(key, val);
            self.skip_whitespace_and_comments()?;
            match self.chars.next() {
                Some(',') => {
                    self.skip_whitespace_and_comments()?;
                    if self.chars.peek() == Some(&'}') {
                        self.chars.next();
                        self.depth -= 1;
                        return Ok(obj);
                    }
                }
                Some('}') => {
                    self.depth -= 1;
                    return Ok(obj);
                }
                _ => return Err(ParsonError::Parse),
            }
        }
    }
}

pub fn parse_string(s: &str) -> Result<Value, ParsonError> {
    // C parson skips UTF-8 BOM: if (string[0]==0xEF && string[1]==0xBB && string[2]==0xBF)
    let s = s.strip_prefix('\u{FEFF}').unwrap_or(s);
    let mut parser = Parser::new(s, false);
    let value = parser.parse_value()?;
    parser.skip_whitespace_and_comments()?;
    if parser.chars.next().is_some() {
        return Err(ParsonError::Parse);
    }
    Ok(value)
}

pub fn parse_string_with_comments(s: &str) -> Result<Value, ParsonError> {
    let s = s.strip_prefix('\u{FEFF}').unwrap_or(s);
    let mut parser = Parser::new(s, true);
    let value = parser.parse_value()?;
    parser.skip_whitespace_and_comments()?;
    if parser.chars.next().is_some() {
        return Err(ParsonError::Parse);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primitives() {
        assert_eq!(parse_string("null"), Ok(Value::Null));
        assert_eq!(parse_string("true"), Ok(Value::Bool(true)));
        assert_eq!(parse_string("false"), Ok(Value::Bool(false)));
        assert_eq!(parse_string("123.45e2"), Ok(Value::Number(12345.0)));
        assert_eq!(parse_string("\"hello world\\t\\n\""), Ok(Value::String("hello world\t\n".to_string())));
    }

    #[test]
    fn test_parse_array() {
        let res = parse_string("[1, 2, \"test\", null]");
        assert!(res.is_ok());
        let val = res.unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr.get(0), Some(&Value::Number(1.0)));
        assert_eq!(arr.get(3), Some(&Value::Null));
    }

    #[test]
    fn test_parse_object() {
        let res = parse_string(r#"{ "name": "Antigravity", "active": true, "count": 10 }"#);
        assert!(res.is_ok());
        let obj = res.unwrap();
        let o = obj.as_object().unwrap();
        assert_eq!(o.get("name"), Some(&Value::String("Antigravity".to_string())));
        assert_eq!(o.get("active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_comments_and_whitespace() {
        let json = r#"
        // Single-line comment before JSON
        /* Block comment
           multiline */
        {
            "key": "val" // Trailing comment
        }
        /* Ending comment */
        "#;
        let res = parse_string_with_comments(json);
        assert!(res.is_ok());
    }

    #[test]
    fn test_trailing_comment_rejected_in_plain_parser() {
        assert_eq!(parse_string("{\"a\":1}//comment"), Err(ParsonError::Parse));
        assert!(parse_string_with_comments("{\"a\":1}//comment").is_ok());
    }

    #[test]
    fn test_invalid_syntax() {
        assert!(parse_string("[1, 2,]").is_ok()); // trailing comma allowed in parson parity
        assert_eq!(parse_string("{ \"a\": }"), Err(ParsonError::Parse));
        assert_eq!(parse_string("null trailing"), Err(ParsonError::Parse));
    }
}
