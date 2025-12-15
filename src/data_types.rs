// File Version: 1.9.0
// /src/data_types.rs

use std::fmt;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Local};
use crate::complex_types;
use crate::user_data::RustcriptObject;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(String),
    Time(SystemTime),
    Tuple(Vec<Value>),
    Vector(Vec<Value>),
    HashMap(HashMap<String, Value>),
    Function(String),
    UserData(Arc<Mutex<dyn RustcriptObject>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Time(a), Value::Time(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::HashMap(a), Value::HashMap(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::UserData(a), Value::UserData(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Value {
    pub fn infer(raw: &str) -> Result<Self, String> {
        let trimmed = raw.trim();

        if let Some(s) = Self::parse_string_literal(trimmed) {
            return Ok(Value::String(s));
        }

        if let Some(complex) = Self::parse_complex_structure(trimmed)? {
            return Ok(complex);
        }

        if let Some(b) = Self::parse_bool(trimmed) {
            return Ok(Value::Boolean(b));
        }

        if let Some(first) = trimmed.chars().next() {
            if first.is_ascii_digit() || first == '-' {
                return Self::parse_number(trimmed);
            }
        }

        Err(format!("Syntax Error: Literal '{}' is invalid. Strings must be quoted.", trimmed))
    }

    pub fn parse_input(raw: &str) -> Self {
        Self::infer(raw).unwrap_or_else(|_| Value::String(raw.trim().to_string()))
    }

    fn parse_string_literal(s: &str) -> Option<String> {
        if s.starts_with("'''") && s.ends_with("'''") && s.len() >= 6 {
            Some(s[3..s.len()-3].to_string())
        } else if s.starts_with('\'') && s.ends_with('\'') {
            Some(s[1..s.len()-1].to_string())
        } else {
            None
        }
    }

    fn parse_bool(s: &str) -> Option<bool> {
        match s {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    fn parse_complex_structure(s: &str) -> Result<Option<Value>, String> {
        if (s.starts_with('(') && s.ends_with(')')) ||
           (s.starts_with('{') && s.ends_with('}')) ||
           (s.starts_with('[') && s.ends_with(']')) {
            complex_types::parse_complex(s).map(Some)
        } else {
            Ok(None)
        }
    }

    fn parse_number(s: &str) -> Result<Value, String> {
        if !s.contains('.') && !s.contains('e') && !s.contains('E') {
            if let Ok(i) = s.parse::<i32>() {
                return Ok(Value::Integer(i));
            }
        }

        match s.parse::<f64>() {
            Ok(f) => {
                if f.is_infinite() {
                    eprintln!("Warning: Float '{}' overflowed. Clamped to MAX.", s);
                    Ok(Value::Float(f64::MAX))
                } else if f.is_nan() {
                    eprintln!("Warning: Float '{}' is NaN. Defaulting to 0.0.", s);
                    Ok(Value::Float(0.0))
                } else {
                    Ok(Value::Float(f))
                }
            },
            Err(e) => {
                if s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E') {
                    Err(format!("Failed to parse float '{}': {}", s, e))
                } else {
                    Err(format!("Invalid number format: {}", s))
                }
            }
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            Value::Integer(_) => "i32".to_string(),
            Value::Float(_) => "f64".to_string(),
            Value::Boolean(_) => "bool".to_string(),
            Value::String(_) => "&str".to_string(),
            Value::Time(_) => "time".to_string(),
            Value::Tuple(_) => "tuple".to_string(),
            Value::Vector(_) => "vector".to_string(),
            Value::HashMap(_) => "hashmap".to_string(),
            Value::Function(_) => "function".to_string(),
            Value::UserData(obj) => {
                obj.lock().map(|g| g.type_name().to_string()).unwrap_or_else(|_| "UserData(Locked)".to_string())
            }
        }
    }

    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            Value::Integer(i) => Ok(*i as f64),
            Value::Float(f) => Ok(*f),
            Value::String(s) => s.parse::<f64>().map_err(|_| "Invalid float string".to_string()),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Time(t) => t.duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .map_err(|_| "Time error".to_string()),
            _ => Err(format!("Cannot coerce {} to Float", self.type_name())),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => s == "true",
            Value::Time(_) | Value::Function(_) | Value::UserData(_) => true,
            Value::Tuple(v) | Value::Vector(v) => !v.is_empty(),
            Value::HashMap(m) => !m.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(val) => write!(f, "{}", val),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Time(t) => {
                let dt: DateTime<Local> = (*t).into();
                write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S"))
            },
            Value::Tuple(vals) => write_collection(f, "(", ")", vals),
            Value::Vector(vals) => write_collection(f, "{", "}", vals),
            Value::HashMap(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            },
            Value::Function(name) => write!(f, "<Function: {}>", name),
            Value::UserData(obj) => {
                if let Ok(guard) = obj.lock() {
                    write!(f, "<{}>", guard.type_name())
                } else {
                    write!(f, "<UserData(Poisoned)>")
                }
            }
        }
    }
}

fn write_collection(f: &mut fmt::Formatter, open: &str, close: &str, items: &[Value]) -> fmt::Result {
    write!(f, "{}", open)?;
    for (i, v) in items.iter().enumerate() {
        if i > 0 { write!(f, ", ")?; }
        write!(f, "{}", v)?;
    }
    write!(f, "{}", close)
}
