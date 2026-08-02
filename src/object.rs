use crate::Value;

/// A JSON object — ordered map of string keys to `Value`s.
///
/// Uses `Vec` of `(String, Value)` pairs rather than `HashMap` to preserve
/// insertion order, matching parson's behaviour.
#[derive(Debug, Clone, PartialEq)]
pub struct Object(pub(crate) Vec<(String, Value)>);

impl Object {
    /// Create an empty object.
    pub fn new() -> Self {
        Object(Vec::new())
    }

    /// Return the number of key-value pairs.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return `true` if the object has no keys.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a reference to the value at `key`, or `None`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Get a mutable reference to the value at `key`, or `None`.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Insert or update a key-value pair.
    pub fn set(&mut self, key: String, value: Value) {
        if let Some(entry) = self.0.iter_mut().find(|(k, _)| k == &key) {
            entry.1 = value;
        } else {
            self.0.push((key, value));
        }
    }

    /// Remove a key. Returns `true` if the key existed.
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(pos) = self.0.iter().position(|(k, _)| k == key) {
            self.0.remove(pos);
            true
        } else {
            false
        }
    }

    /// Return the key at `index`, or `None`.
    pub fn get_key_at(&self, index: usize) -> Option<&str> {
        self.0.get(index).map(|(k, _)| k.as_str())
    }

    /// Return the value at `index`, or `None`.
    pub fn get_value_at(&self, index: usize) -> Option<&Value> {
        self.0.get(index).map(|(_, v)| v)
    }

    /// Dot-notation lookup: `"a.b.c"` resolves nested objects.
    pub fn get_dotted(&self, dotkey: &str) -> Option<&Value> {
        let mut parts = dotkey.splitn(2, '.');
        let first = parts.next()?;
        let val = self.get(first)?;
        if let Some(rest) = parts.next() {
            if let Value::Object(obj) = val {
                obj.get_dotted(rest)
            } else {
                None
            }
        } else {
            Some(val)
        }
    }

    /// Alias for `get_dotted`, returning nested property or `None`.
    pub fn dotget(&self, dotkey: &str) -> Option<&Value> {
        self.get_dotted(dotkey)
    }

    /// Iterate over key-value pairs.
    pub fn iter(&self) -> std::slice::Iter<'_, (String, Value)> {
        self.0.iter()
    }
}

impl Default for Object {
    fn default() -> Self {
        Object::new()
    }
}
