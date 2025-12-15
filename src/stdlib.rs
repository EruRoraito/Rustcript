// File Version: 2.8.1
// /src/stdlib.rs

use crate::data_types::Value;
use crate::json_lib;
use crate::types::IoPermissions;
use std::time::UNIX_EPOCH;
use std::path::Path;
use chrono::{DateTime, Local};
use rand::seq::SliceRandom;
use rand::Rng;

#[cfg(feature = "os_access")]
use std::process::Command;

#[cfg(feature = "file_io")]
use crate::io_lib;


fn check_args(args: &[Value], count: usize, method: &str) -> Result<(), String> {
    if args.len() != count {
        Err(format!("{} expects {} arguments, got {}", method, count, args.len()))
    } else {
        Ok(())
    }
}


fn handle_math(method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
    match method {
        "sqrt" => {
            check_args(&args, 1, "math.sqrt")?;
            let f = args[0].as_float().map_err(|_| "Argument must be a number")?;
            Ok(Some(Value::Float(f.sqrt())))
        },
        "pow" => {
            check_args(&args, 2, "math.pow")?;
            let base = args[0].as_float().map_err(|_| "Base must be a number")?;
            let exp = args[1].as_float().map_err(|_| "Exponent must be a number")?;
            Ok(Some(Value::Float(base.powf(exp))))
        },
        "abs" => {
            check_args(&args, 1, "math.abs")?;
            let f = args[0].as_float().map_err(|_| "Argument must be a number")?;
            Ok(Some(Value::Float(f.abs())))
        },
        "round" => {
            check_args(&args, 1, "math.round")?;
            let f = args[0].as_float().map_err(|_| "Argument must be a number")?;
            Ok(Some(Value::Integer(f.round() as i32)))
        },
        "floor" => {
            check_args(&args, 1, "math.floor")?;
            let f = args[0].as_float().map_err(|_| "Argument must be a number")?;
            Ok(Some(Value::Integer(f.floor() as i32)))
        },
        "ceil" => {
            check_args(&args, 1, "math.ceil")?;
            let f = args[0].as_float().map_err(|_| "Argument must be a number")?;
            Ok(Some(Value::Integer(f.ceil() as i32)))
        },
        "sin" => {
            check_args(&args, 1, "math.sin")?;
            let f = args[0].as_float().map_err(|_| "Argument must be a number")?;
            Ok(Some(Value::Float(f.sin())))
        },
        "cos" => {
            check_args(&args, 1, "math.cos")?;
            let f = args[0].as_float().map_err(|_| "Argument must be a number")?;
            Ok(Some(Value::Float(f.cos())))
        },
        "pi" => Ok(Some(Value::Float(std::f64::consts::PI))),
        "e" => Ok(Some(Value::Float(std::f64::consts::E))),
        _ => Err(format!("Unknown method '{}' for math module", method)),
    }
}

fn handle_rand(method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
    let mut rng = rand::rng();
    match method {
        "int" => {
            check_args(&args, 2, "rand.int")?;
            let min = args[0].as_float().map_err(|_| "Min must be a number")? as i32;
            let max = args[1].as_float().map_err(|_| "Max must be a number")? as i32;
            if min >= max { return Err("Min must be less than Max".to_string()); }
            Ok(Some(Value::Integer(rng.random_range(min..max))))
        },
        "float" => Ok(Some(Value::Float(rng.random::<f64>()))),
        "bool" => Ok(Some(Value::Boolean(rng.random::<bool>()))),
        _ => Err(format!("Unknown method '{}' for rand module", method)),
    }
}

fn handle_json(method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
    match method {
        "parse" => {
            check_args(&args, 1, "json.parse")?;
            let json_str = args[0].to_string();
            let val = json_lib::parse(&json_str)?;
            Ok(Some(val))
        },
        "stringify" => {
            if args.is_empty() { return Err("json.stringify expects at least 1 argument".to_string()); }
            let pretty = if args.len() > 1 { args[1].as_bool() } else { false };
            let s = json_lib::stringify(&args[0], pretty)?;
            Ok(Some(Value::String(s)))
        },
        _ => Err(format!("Unknown method '{}' for json module", method)),
    }
}


pub fn call_static(
    module: &str,
    method: &str,
    args: Vec<Value>,
    _sandbox_root: Option<&Path>,
    _io_perms: &IoPermissions
) -> Result<Option<Value>, String> {
    match module {
        "math" => handle_math(method, args),
        "rand" => handle_rand(method, args),
        "json" => handle_json(method, args),
        "os" => {
            #[cfg(not(feature = "os_access"))]
            { return Err("Security Violation: 'os' module is disabled.".to_string()); }

            #[cfg(feature = "os_access")]
            {
                if method == "exec" {
                    check_args(&args, 1, "os.exec")?;
                    let cmd_raw = args[0].to_string();
                    let parts: Vec<&str> = cmd_raw.split_whitespace().collect();
                    if parts.is_empty() { return Ok(Some(Value::Integer(-1))); }

                    let output_res = Command::new(parts[0]).args(&parts[1..]).output();
                    match output_res {
                        Ok(output) => Ok(Some(Value::Integer(output.status.code().unwrap_or(-1)))),
                        Err(_) => Ok(Some(Value::Integer(-1)))
                    }
                } else {
                    Err(format!("Unknown method '{}' for os module", method))
                }
            }
        },
        "io" => {
            #[cfg(not(feature = "file_io"))]
            { return Err("Security Violation: 'io' module is disabled.".to_string()); }

            #[cfg(feature = "file_io")]
            {
                io_lib::handle_io(_sandbox_root, _io_perms, method, args)
            }
        },
        _ => Err(format!("Unknown static module '{}'", module)),
    }
}


fn method_vector(vec: &mut Vec<Value>, method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
    match method {
        "push" => {
            check_args(&args, 1, "push")?;
            vec.push(args[0].clone());
            Ok(None)
        },
        "pop" => Ok(Some(vec.pop().ok_or("Cannot pop from empty vector")?)),
        "len" => Ok(Some(Value::Integer(vec.len() as i32))),
        "get" => {
            check_args(&args, 1, "get")?;
            let idx = args[0].as_float().map_err(|_| "Index must be number")? as usize;
            Ok(Some(vec.get(idx).ok_or("Index out of bounds")?.clone()))
        },
        "remove" => {
             check_args(&args, 1, "remove")?;
             let idx = args[0].as_float().map_err(|_| "Index must be number")? as usize;
             if idx >= vec.len() { return Err("Index out of bounds".to_string()); }
             Ok(Some(vec.remove(idx)))
        },
        "insert" => {
             check_args(&args, 2, "insert")?;
             let idx = args[0].as_float().map_err(|_| "Index must be number")? as usize;
             if idx > vec.len() { return Err("Index out of bounds".to_string()); }
             vec.insert(idx, args[1].clone());
             Ok(None)
        },
        "clear" => { vec.clear(); Ok(None) },
        "join" => {
            check_args(&args, 1, "join")?;
            let sep = args[0].to_string();
            let strings: Vec<String> = vec.iter().map(|v| v.to_string()).collect();
            Ok(Some(Value::String(strings.join(&sep))))
        },
        "shuffle" => {
            vec.shuffle(&mut rand::rng());
            Ok(None)
        },
        _ => Err(format!("Unknown method '{}' for Vector", method)),
    }
}

fn method_string(s: &str, method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
    match method {
        "len" => Ok(Some(Value::Integer(s.chars().count() as i32))),
        "to_upper" => Ok(Some(Value::String(s.to_uppercase()))),
        "to_lower" => Ok(Some(Value::String(s.to_lowercase()))),
        "trim" => Ok(Some(Value::String(s.trim().to_string()))),
        "trim_start" => Ok(Some(Value::String(s.trim_start().to_string()))),
        "trim_end" => Ok(Some(Value::String(s.trim_end().to_string()))),
        "contains" => {
            check_args(&args, 1, "contains")?;
            Ok(Some(Value::Boolean(s.contains(&args[0].to_string()))))
        },
        "starts_with" => {
            check_args(&args, 1, "starts_with")?;
            Ok(Some(Value::Boolean(s.starts_with(&args[0].to_string()))))
        },
        "ends_with" => {
            check_args(&args, 1, "ends_with")?;
            Ok(Some(Value::Boolean(s.ends_with(&args[0].to_string()))))
        },
        "replace" => {
            check_args(&args, 2, "replace")?;
            Ok(Some(Value::String(s.replace(&args[0].to_string(), &args[1].to_string()))))
        },
        "split" => {
            check_args(&args, 1, "split")?;
            let delim = args[0].to_string();
            let parts = s.split(&delim).map(|sub| Value::String(sub.to_string())).collect();
            Ok(Some(Value::Vector(parts)))
        },
        "index_of" => {
             check_args(&args, 1, "index_of")?;
             let idx = s.find(&args[0].to_string()).map(|i| i as i32).unwrap_or(-1);
             Ok(Some(Value::Integer(idx)))
        },
        "substring" => {
            check_args(&args, 2, "substring")?;
            let start = args[0].as_float().unwrap_or(0.0) as usize;
            let end = args[1].as_float().unwrap_or(0.0) as usize;
            if start > end { return Err("Start index cannot be greater than end index".to_string()); }
            let sub: String = s.chars().skip(start).take(end - start).collect();
            Ok(Some(Value::String(sub)))
        },
        "to_int" => {
            let i = s.trim().parse::<i32>().map_err(|_| "Cannot parse to Integer".to_string())?;
            Ok(Some(Value::Integer(i)))
        },
        "to_float" => {
             let f = s.trim().parse::<f64>().map_err(|_| "Cannot parse to Float".to_string())?;
             Ok(Some(Value::Float(f)))
        },
        "is_match" | "find_all" | "regex_replace" => {
            crate::regex_lib::handle_method(s, method, args)
        },
        _ => Err(format!("Unknown method '{}' for String", method)),
    }
}

pub fn call_method(obj: &mut Value, method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {

    if let Some(dot_idx) = method.find('.') {
        let prop = &method[..dot_idx];
        let next_method = &method[dot_idx+1..];

        return match obj {
             Value::HashMap(map) => {
                 let sub = map.get_mut(prop).ok_or_else(|| format!("Property '{}' not found", prop))?;
                 call_method(sub, next_method, args)
             },
             Value::Vector(vec) | Value::Tuple(vec) => {
                 let idx = prop.parse::<usize>().map_err(|_| "Index must be number".to_string())?;
                 let sub = vec.get_mut(idx).ok_or("Index out of bounds")?;
                 call_method(sub, next_method, args)
             },
             Value::UserData(user_obj) => {
                 let guard = user_obj.lock().map_err(|_| "UserData poisoned".to_string())?;
                 if let Some(mut val) = guard.get(prop) {
                     call_method(&mut val, next_method, args)
                 } else {
                     Err(format!("Property '{}' not found", prop))
                 }
             },
             _ => Err(format!("Cannot traverse property on type {}", obj.type_name())),
        };
    }

    match obj {
        Value::UserData(user_obj) => {
             user_obj.lock().map_err(|_| "UserData poisoned".to_string())?.call(method, args)
        },
        Value::Vector(vec) => method_vector(vec, method, args),
        Value::HashMap(map) => match method {
            "insert" => {
                check_args(&args, 2, "insert")?;
                map.insert(args[0].to_string(), args[1].clone());
                Ok(None)
            },
            "remove" => {
                check_args(&args, 1, "remove")?;
                let val = map.remove(&args[0].to_string()).ok_or("Key not found")?;
                Ok(Some(val))
            },
            "get" => {
                check_args(&args, 1, "get")?;
                let val = map.get(&args[0].to_string()).ok_or("Key not found")?;
                Ok(Some(val.clone()))
            },
            "len" => Ok(Some(Value::Integer(map.len() as i32))),
            "contains" => {
                check_args(&args, 1, "contains")?;
                Ok(Some(Value::Boolean(map.contains_key(&args[0].to_string()))))
            },
            "keys" => {
                let keys = map.keys().map(|k| Value::String(k.clone())).collect();
                Ok(Some(Value::Vector(keys)))
            },
            _ => Err(format!("Unknown method '{}' for HashMap", method)),
        },
        Value::Tuple(vec) => {
             if method == "len" { Ok(Some(Value::Integer(vec.len() as i32))) }
             else { Err(format!("Unknown method '{}' for Tuple", method)) }
        },
        Value::String(s) => method_string(s, method, args),
        Value::Time(t) => match method {
            "elapsed" => Ok(Some(Value::Float(t.elapsed().map_err(|_| "Time error")?.as_secs_f64()))),
            "timestamp" => Ok(Some(Value::Integer(t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i32))),
            "date" => {
                let dt: DateTime<Local> = (*t).into();
                Ok(Some(Value::String(dt.format("%Y-%m-%d").to_string())))
            },
            "time" => {
                let dt: DateTime<Local> = (*t).into();
                Ok(Some(Value::String(dt.format("%H:%M:%S").to_string())))
            },
            _ => Err(format!("Unknown method '{}' for Time", method)),
        },
        _ => Err(format!("Type {} does not support methods", obj.type_name())),
    }
}
