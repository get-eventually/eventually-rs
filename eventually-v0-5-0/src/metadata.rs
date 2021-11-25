use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Metadata(HashMap<String, Value>);

impl Metadata {
    pub fn add_string(mut self, key: String, value: String) -> Self {
        self.0.insert(key, Value::String(value));
        self
    }

    pub fn add_float(mut self, key: String, value: f64) -> Self {
        self.0.insert(key, Value::Number(value));
        self
    }

    pub fn add_integer(self, key: String, value: i64) -> Self {
        self.add_float(key, value as f64)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, Value)> {
        self.0.into_iter()
    }
}
