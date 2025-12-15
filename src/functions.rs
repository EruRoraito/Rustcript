//  File Version: 1.3.1
//  /src/functions.rs

use crate::data_types::Value;
use std::collections::HashMap;

fn strip_quotes(s: &str) -> &str {
    let trimmed = s.trim();
    if trimmed.starts_with("'''") && trimmed.ends_with("'''") && trimmed.len() >= 6 {
        return &trimmed[3..trimmed.len()-3];
    }
    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
        return &trimmed[1..trimmed.len()-1];
    }
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return &trimmed[1..trimmed.len()-1];
    }
    trimmed
}

pub fn parse_definition(raw: &str) -> Result<(String, Vec<String>), String> {
    let trimmed = strip_quotes(raw);
    if trimmed.is_empty() {
        return Err("Function definition cannot be empty".to_string());
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Function missing name".to_string());
    }

    let name = parts[0].to_string();
    let mut args = Vec::new();

    for i in 1..parts.len() {
        args.push(parts[i].to_string());
    }

    Ok((name, args))
}

pub fn parse_call(raw: &str) -> Result<(Option<String>, String, Vec<String>), String> {
    let inner = strip_quotes(raw);

    let (target, rest) = if let Some(eq_idx) = inner.find('=') {
        let t = inner[..eq_idx].trim().to_string();
        let r = inner[eq_idx+1..].trim();
        (Some(t), r)
    } else {
        (None, inner)
    };

    let paren_open = rest.find('(').ok_or("Function call requires '('")?;
    let paren_close = rest.rfind(')').ok_or("Function call requires ')'")?;

    let func_name = rest[..paren_open].trim().to_string();

    if func_name.is_empty() {
        return Err("Function name cannot be empty".to_string());
    }

    let args_str = &rest[paren_open+1..paren_close];

    let args = if args_str.trim().is_empty() {
        Vec::new()
    } else {
        crate::parser::split_args(args_str)
    };

    Ok((target, func_name, args))
}

pub fn bind_arguments(
    locals: &mut HashMap<String, Value>,
    param_names: &[String],
    arg_values: Vec<Value>
) -> Result<(), String> {
    if arg_values.len() != param_names.len() {
        return Err(format!(
            "Argument mismatch: Expected {}, got {}",
            param_names.len(),
            arg_values.len()
        ));
    }

    for (name, val) in param_names.iter().zip(arg_values.into_iter()) {
        locals.insert(name.clone(), val);
    }

    Ok(())
}
