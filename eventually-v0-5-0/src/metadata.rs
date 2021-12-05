use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata(HashMap<String, Value>);

impl Metadata {
    #[must_use]
    pub fn add(mut self, key: String, value: Value) -> Self {
        self.0.insert(key, value);
        self
    }

    #[must_use]
    pub fn add_string(self, key: String, value: String) -> Self {
        self.add(key, Value::String(value))
    }

    #[must_use]
    pub fn add_float(self, key: String, value: f64) -> Self {
        self.add(key, Value::Number(value))
    }

    #[must_use]
    pub fn add_boolean(self, key: String, value: bool) -> Self {
        self.add(key, Value::Boolean(value))
    }

    #[must_use]
    pub fn add_integer(self, key: String, value: i32) -> Self {
        self.add_float(key, f64::from(value))
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
    }
}

impl IntoIterator for Metadata {
    type Item = (String, Value);
    type IntoIter = std::collections::hash_map::IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_serializes_correctly() {
        let metadata = Metadata::default()
            .add_string("string".to_owned(), "test".to_owned())
            .add_float("float".to_owned(), std::f64::consts::PI)
            .add_integer("integer".to_owned(), 1)
            .add_boolean("boolean".to_owned(), false);

        let serialized =
            serde_json::to_string_pretty(&metadata).expect("metadata should be serialized");

        let deserialized = serde_json::from_str(&serialized)
            .expect("metadata should be deserialized from its serialized form");

        assert_eq!(metadata, deserialized);
    }
}
