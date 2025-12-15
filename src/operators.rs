// File Version: 2.6.0
// /src/operators.rs

use crate::data_types::Value;
use std::time::Duration;

pub fn perform_arithmetic(left: &Value, op: &str, right: &Value) -> Result<Value, String> {
    if ["==", "!=", ">", "<", ">=", "<="].contains(&op) {
        let bool_res = perform_comparison(left, op, right)?;
        return Ok(Value::Boolean(bool_res));
    }

    if ["&&", "||"].contains(&op) {
        let bool_res = perform_logic(left, op, right)?;
        return Ok(Value::Boolean(bool_res));
    }

    if let Value::Time(t) = left {
        if op == "+" {
             let seconds = right.as_float().map_err(|_| "Can only add numbers (seconds) to Time")?;
             let duration = Duration::from_secs_f64(seconds);
             return Ok(Value::Time(*t + duration));
        } else if op == "-" {
             if let Value::Time(t2) = right {
                 return match t.duration_since(*t2) {
                     Ok(d) => Ok(Value::Float(d.as_secs_f64())),
                     Err(e) => {
                         let d = e.duration();
                         Ok(Value::Float(-(d.as_secs_f64())))
                     }
                 };
             }
             let seconds = right.as_float().map_err(|_| "Can only subtract numbers (seconds) from Time")?;
             let duration = Duration::from_secs_f64(seconds);
             return Ok(Value::Time(*t - duration));
        }
    }

    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => match op {
            "+" => Ok(Value::Integer(l + r)),
            "-" => Ok(Value::Integer(l - r)),
            "*" => Ok(Value::Integer(l * r)),
            "/" => if *r == 0 { Err("Division by zero".to_string()) } else { Ok(Value::Integer(l / r)) },
            "%" => if *r == 0 { Err("Modulo by zero".to_string()) } else { Ok(Value::Integer(l % r)) },
            _ => Err(format!("Unknown int operator: {}", op)),
        },
        (l_val, r_val) => {
            if let (Value::String(s1), Value::String(s2)) = (l_val, r_val) {
                match op {
                    "+" => return Ok(Value::String(format!("{}{}", s1, s2))),
                    _ => return Err(format!("Strings do not support operator: {}", op)),
                }
            }

            let l = l_val.as_float().map_err(|_| format!("Cannot convert {} to float", l_val.type_name()))?;
            let r = r_val.as_float().map_err(|_| format!("Cannot convert {} to float", r_val.type_name()))?;

            match op {
                "+" => Ok(Value::Float(l + r)),
                "-" => Ok(Value::Float(l - r)),
                "*" => Ok(Value::Float(l * r)),
                "/" => if r == 0.0 { Err("Division by zero".to_string()) } else { Ok(Value::Float(l / r)) },
                "%" => if r == 0.0 { Err("Modulo by zero".to_string()) } else { Ok(Value::Float(l % r)) },
                _ => Err(format!("Unknown float operator: {}", op)),
            }
        }
    }
}

pub fn perform_assignment(current: &Value, op: &str, operand: &Value) -> Result<Value, String> {
    match op {
        "=" => Ok(operand.clone()),
        "+=" => perform_arithmetic(current, "+", operand),
        "-=" => perform_arithmetic(current, "-", operand),
        "*=" => perform_arithmetic(current, "*", operand),
        "/=" => perform_arithmetic(current, "/", operand),
        "%=" => perform_arithmetic(current, "%", operand),
        _ => Err(format!("Unknown assignment operator: {}", op)),
    }
}

pub fn perform_comparison(left: &Value, op: &str, right: &Value) -> Result<bool, String> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => match op {
            "==" => Ok(l == r),
            "!=" => Ok(l != r),
            ">"  => Ok(l > r),
            "<"  => Ok(l < r),
            ">=" => Ok(l >= r),
            "<=" => Ok(l <= r),
            _ => Err(format!("Unknown comparison op: {}", op)),
        },
        (Value::Boolean(l), Value::Boolean(r)) => match op {
            "==" => Ok(l == r),
            "!=" => Ok(l != r),
            _ => Err("Booleans only support == and !=".to_string()),
        },
        (Value::String(l), Value::String(r)) => match op {
            "==" => Ok(l == r),
            "!=" => Ok(l != r),
            _ => Err("Strings only support == and !=".to_string()),
        },
        (Value::Time(l), Value::Time(r)) => match op {
            "==" => Ok(l == r),
            "!=" => Ok(l != r),
            ">"  => Ok(l > r),
            "<"  => Ok(l < r),
            ">=" => Ok(l >= r),
            "<=" => Ok(l <= r),
            _ => Err(format!("Unknown comparison op for Time: {}", op)),
        },
        (Value::Function(l), Value::Function(r)) => match op {
            "==" => Ok(l == r),
            "!=" => Ok(l != r),
            _ => Err("Functions only support == and !=".to_string()),
        },
        (l_val, r_val) => {
            let l = l_val.as_float().unwrap_or(0.0);
            let r = r_val.as_float().unwrap_or(0.0);
            match op {
                "==" => Ok((l - r).abs() < f64::EPSILON),
                "!=" => Ok((l - r).abs() > f64::EPSILON),
                ">"  => Ok(l > r),
                "<"  => Ok(l < r),
                ">=" => Ok(l >= r),
                "<=" => Ok(l <= r),
                _ => Err(format!("Cannot compare {} and {}", l_val.type_name(), r_val.type_name())),
            }
        }
    }
}

pub fn perform_logic(left: &Value, op: &str, right: &Value) -> Result<bool, String> {
    let l_bool = left.as_bool();
    let r_bool = right.as_bool();
    match op {
        "&&" => Ok(l_bool && r_bool),
        "||" => Ok(l_bool || r_bool),
        _ => Err(format!("Unknown logic op: {}", op)),
    }
}

pub fn perform_unary_logic(op: &str, val: &Value) -> Result<bool, String> {
    match op {
        "!" => Ok(!val.as_bool()),
        _ => Err(format!("Unknown unary op: {}", op)),
    }
}
