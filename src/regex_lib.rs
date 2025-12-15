// File Version: 1.0.0
// /src/regex_lib.rs

use crate::data_types::Value;
use regex::Regex;

pub fn handle_method(s: &str, method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
    match method {
        "is_match" => {
            if args.len() != 1 { return Err("is_match expects 1 argument (regex_pattern)".to_string()); }
            let pattern = args[0].to_string();
            let re = Regex::new(&pattern).map_err(|e| format!("Invalid Regex: {}", e))?;
            Ok(Some(Value::Boolean(re.is_match(s))))
        },
        "find_all" => {
            if args.len() != 1 { return Err("find_all expects 1 argument (regex_pattern)".to_string()); }
            let pattern = args[0].to_string();
            let re = Regex::new(&pattern).map_err(|e| format!("Invalid Regex: {}", e))?;
            let matches: Vec<Value> = re.find_iter(s)
                .map(|m| Value::String(m.as_str().to_string()))
                .collect();
            Ok(Some(Value::Vector(matches)))
        },
        "regex_replace" => {
            if args.len() != 2 { return Err("regex_replace expects 2 arguments (pattern, replacement)".to_string()); }
            let pattern = args[0].to_string();
            let replacement = args[1].to_string();
            let re = Regex::new(&pattern).map_err(|e| format!("Invalid Regex: {}", e))?;
            let result = re.replace_all(s, replacement.as_str());
            Ok(Some(Value::String(result.to_string())))
        },
        _ => Err(format!("Unknown regex method '{}'", method)),
    }
}
