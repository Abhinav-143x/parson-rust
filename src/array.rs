use crate::Value;

/// A JSON array — ordered list of `Value`s.
///
/// Wraps a `Vec<Value>` and provides parson-compatible index access.
#[derive(Debug, Clone, PartialEq)]
pub struct Array(pub(crate) Vec<Value>);

impl Array {
    /// Create an empty array.
    pub fn new() -> Self {
        Array(Vec::new())
    }

    /// Return the number of items.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return `true` if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a reference to the item at `index`, or `None`.
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Append a value.
    pub fn push(&mut self, value: Value) {
        self.0.push(value);
    }

    /// Remove the item at `index`. Returns `true` on success.
    pub fn remove(&mut self, index: usize) -> bool {
        if index < self.0.len() {
            self.0.remove(index);
            true
        } else {
            false
        }
    }

    /// Iterate over contained values.
    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.0.iter()
    }
}

impl Default for Array {
    fn default() -> Self {
        Array::new()
    }
}

impl FromIterator<Value> for Array {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Array(iter.into_iter().collect())
    }
}
