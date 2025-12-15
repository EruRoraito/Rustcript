// File Version: 1.5.1
// /src/interpreter_utils.rs

use crate::data_types::Value;

#[derive(Debug)]
pub enum AccessOp {
    Dot(String),
    Bracket(String),
}

pub fn parse_access_chain(raw: &str) -> (String, Vec<AccessOp>) {
    let mut root = String::new();
    let mut ops = Vec::new();
    let mut chars = raw.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '.' || c == '[' { break; }
        root.push(c);
        chars.next();
    }

    while let Some(&c) = chars.peek() {
        match c {
            '.' => {
                chars.next();
                let mut prop = String::new();
                while let Some(&n) = chars.peek() {
                    if n == '.' || n == '[' { break; }
                    prop.push(n);
                    chars.next();
                }
                if !prop.is_empty() { ops.push(AccessOp::Dot(prop)); }
            },
            '[' => {
                chars.next();
                let mut expr = String::new();
                let mut depth = 1;
                while let Some(n) = chars.next() {
                    match n {
                        '[' => depth += 1,
                        ']' => depth -= 1,
                        _ => {}
                    }
                    if depth == 0 { break; }
                    expr.push(n);
                }
                ops.push(AccessOp::Bracket(expr));
            },
            _ => { chars.next(); } 
        }
    }

    (root, ops)
}

pub fn access_property(val: &Value, prop: &str) -> Option<Value> {
    match val {
        Value::Tuple(vec) | Value::Vector(vec) => {
            let idx = prop.parse::<usize>().ok()?;
            vec.get(idx).cloned()
        },
        Value::HashMap(map) => map.get(prop).cloned(),
        Value::UserData(obj) => {
             obj.lock().ok()?.get(prop)
        },
        _ => None
    }
}

pub fn access_dynamic(val: &Value, index: Value) -> Option<Value> {
    match val {
        Value::Vector(vec) | Value::Tuple(vec) => {
            let idx = index.as_float().ok()? as usize;
            vec.get(idx).cloned()
        },
        Value::HashMap(map) => {
            map.get(&index.to_string()).cloned()
        },
        Value::UserData(obj) => {
            obj.lock().ok()?.get(&index.to_string())
        },
        _ => None
    }
}

pub fn mutate_chain(val: &mut Value, keys: Vec<Value>, new_val: Value) -> Result<(), String> {
    if keys.is_empty() { return Ok(()); }

    let (current_key, rest_keys) = (&keys[0], &keys[1..]);
    let is_final = rest_keys.is_empty();

    match val {
        Value::Vector(vec) | Value::Tuple(vec) => {
            let idx = current_key.as_float().map_err(|_| "Index must be number")? as usize;
            if idx >= vec.len() {
                return Err(format!("Index {} out of bounds (len {})", idx, vec.len()));
            }

            if is_final {
                vec[idx] = new_val;
                Ok(())
            } else {
                mutate_chain(&mut vec[idx], rest_keys.to_vec(), new_val)
            }
        },
        Value::HashMap(map) => {
            let k_str = current_key.to_string();

            if is_final {
                map.insert(k_str, new_val);
                Ok(())
            } else {
                let sub_val = map.get_mut(&k_str).ok_or_else(|| format!("Key '{}' not found", k_str))?;
                mutate_chain(sub_val, rest_keys.to_vec(), new_val)
            }
        },
        Value::UserData(obj) => {
            let k_str = current_key.to_string();
            if is_final {
                let mut guard = obj.lock().map_err(|_| "UserData poisoned")?;
                guard.set(&k_str, new_val)?;
                Ok(())
            } else {
                Err("Deep mutation on Native Objects (UserData) is not automatically supported. Use methods.".to_string())
            }
        },
        _ => Err(format!("Cannot mutate property on type {}", val.type_name())),
    }
}
