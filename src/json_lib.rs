// File Version: 1.2.0
// /src/json_lib.rs

use crate::data_types::Value;
use serde_json::{Value as JsonValue, Map, Number};
use std::collections::HashMap;
use chrono::{DateTime, Local};

pub fn parse(json_str: &str) -> Result<Value, String> {
    let v: JsonValue = serde_json::from_str(json_str).map_err(|e| format!("JSON Parse Error: {}", e))?;
    Ok(json_to_rustcript(v))
}

pub fn stringify(val: &Value, pretty: bool) -> Result<String, String> {
    let v = rustcript_to_json(val)?;
    if pretty {
        serde_json::to_string_pretty(&v).map_err(|e| format!("JSON Stringify Error: {}", e))
    } else {
        serde_json::to_string(&v).map_err(|e| format!("JSON Stringify Error: {}", e))
    }
}

fn json_to_rustcript(json: JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::String("null".to_string()),
        JsonValue::Bool(b) => Value::Boolean(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    Value::Integer(i as i32)
                } else {
                    Value::Float(n.as_f64().unwrap_or(0.0))
                }
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        },
        JsonValue::String(s) => Value::String(s),
        JsonValue::Array(arr) => {
            let vec: Vec<Value> = arr.into_iter().map(json_to_rustcript).collect();
            Value::Vector(vec)
        },
        JsonValue::Object(map) => {
            let mut hmap = HashMap::new();
            for (k, v) in map {
                hmap.insert(k, json_to_rustcript(v));
            }
            Value::HashMap(hmap)
        }
    }
}

fn rustcript_to_json(val: &Value) -> Result<JsonValue, String> {
    match val {
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Number(Number::from(*i))),
        Value::Float(f) => {
            let n = Number::from_f64(*f).ok_or_else(|| "Infinite or NaN floats cannot be serialized to JSON".to_string())?;
            Ok(JsonValue::Number(n))
        },
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::Time(t) => {
            let dt: DateTime<Local> = (*t).into();
            Ok(JsonValue::String(dt.to_rfc3339()))
        },
        Value::Vector(vec) | Value::Tuple(vec) => {
            let mut arr = Vec::new();
            for item in vec {
                arr.push(rustcript_to_json(item)?);
            }
            Ok(JsonValue::Array(arr))
        },
        Value::HashMap(map) => {
            let mut obj = Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), rustcript_to_json(v)?);
            }
            Ok(JsonValue::Object(obj))
        },
        Value::Function(name) => {
            Ok(JsonValue::String(format!("<Function: {}>", name)))
        },
        Value::UserData(u) => {
            if let Ok(guard) = u.lock() {
                Ok(JsonValue::String(format!("<UserData: {}>", guard.type_name())))
            } else {
                Ok(JsonValue::String("<UserData: Poisoned>".to_string()))
            }
        }
    }
}
