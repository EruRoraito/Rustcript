// File Version: 1.0.0
// /src/user_data.rs

use crate::data_types::Value;
use std::fmt::Debug;

pub trait RustcriptObject: Send + Sync + Debug {
    fn get(&self, _field: &str) -> Option<Value> {
        None
    }

    fn set(&mut self, _field: &str, _value: Value) -> Result<(), String> {
        Err("Property is read-only or does not exist".to_string())
    }

    fn call(&mut self, _method: &str, _args: Vec<Value>) -> Result<Option<Value>, String> {
        Err(format!("Method '{}' not found or not implemented", _method))
    }

    fn type_name(&self) -> &str {
        "UserData"
    }
}
