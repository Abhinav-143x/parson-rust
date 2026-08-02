use crate::{ParsonError, Value};

fn same_type(a: &Value, b: &Value) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// Validates that `value` matches the structure defined in `schema`.
/// Ported from `json_validate` in `parson.c`.
pub fn validate(schema: &Value, value: &Value) -> Result<(), ParsonError> {
    if schema.is_null() {
        return Ok(()); // Null schema validates anything
    }

    if !same_type(schema, value) {
        return Err(ParsonError::TypeError);
    }

    match schema {
        Value::Array(schema_arr) => {
            let val_arr = match value { Value::Array(a) => a, _ => unreachable!() };
            if schema_arr.is_empty() {
                return Ok(()); // Empty array schema validates any array
            }
            let schema_template = schema_arr.get(0).unwrap();
            for item in val_arr.iter() {
                validate(schema_template, item)?;
            }
        }
        Value::Object(schema_obj) => {
            let val_obj = match value { Value::Object(o) => o, _ => unreachable!() };
            for (key, schema_val) in schema_obj.iter() {
                if let Some(val) = val_obj.get(key) {
                    validate(schema_val, val)?;
                } else {
                    return Err(ParsonError::NotFound);
                }
            }
        }
        _ => {} // Primitive types only require variant equivalence
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Object;

    #[test]
    fn test_validate_primitives() {
        assert!(validate(&Value::Null, &Value::Number(42.0)).is_ok());
        assert!(validate(&Value::Number(1.0), &Value::Number(2.0)).is_ok());
        assert!(validate(&Value::Number(1.0), &Value::String("wrong".to_string())).is_err());
    }

    #[test]
    fn test_validate_object_and_array() {
        let mut schema_obj = Object::new();
        schema_obj.set("name".to_string(), Value::String("".to_string()));
        schema_obj.set("age".to_string(), Value::Number(0.0));

        let mut valid_obj = Object::new();
        valid_obj.set("name".to_string(), Value::String("Alice".to_string()));
        valid_obj.set("age".to_string(), Value::Number(30.0));

        let mut invalid_obj = Object::new();
        invalid_obj.set("name".to_string(), Value::String("Bob".to_string()));

        assert!(validate(&Value::Object(schema_obj.clone()), &Value::Object(valid_obj)).is_ok());
        assert_eq!(
            validate(&Value::Object(schema_obj), &Value::Object(invalid_obj)),
            Err(ParsonError::NotFound)
        );
    }
}
