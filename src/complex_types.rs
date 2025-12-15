// File Version: 1.6.0
// /src/complex_types.rs

use crate::data_types::Value;
use std::collections::HashMap;

struct ParseState {
    in_quote: bool,
    is_triple_quote: bool,
    parens: i32,
    braces: i32,
    brackets: i32,
}

impl ParseState {
    fn new() -> Self {
        Self { in_quote: false, is_triple_quote: false, parens: 0, braces: 0, brackets: 0 }
    }

    fn consume(&mut self, c: char, next1: Option<char>, next2: Option<char>) -> usize {
        if c == '\'' && next1 == Some('\'') && next2 == Some('\'') {
            if self.in_quote {
                if self.is_triple_quote {
                    self.in_quote = false;
                    self.is_triple_quote = false;
                    return 3;
                }
            } else {
                self.in_quote = true;
                self.is_triple_quote = true;
                return 3;
            }
        }

        if c == '\'' {
            if self.in_quote {
                if !self.is_triple_quote {
                    self.in_quote = false;
                }
            } else {
                self.in_quote = true;
                self.is_triple_quote = false;
            }
            return 1;
        }

        if !self.in_quote {
            match c {
                '(' => self.parens += 1,
                ')' => if self.parens > 0 { self.parens -= 1 },
                '{' => self.braces += 1,
                '}' => if self.braces > 0 { self.braces -= 1 },
                '[' => self.brackets += 1,
                ']' => if self.brackets > 0 { self.brackets -= 1 },
                _ => {}
            }
        }

        1
    }

    fn is_top_level(&self) -> bool {
        !self.in_quote && self.parens == 0 && self.braces == 0 && self.brackets == 0
    }
}


pub fn parse_complex(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();

    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        let content = &trimmed[1..trimmed.len()-1];
        let items = split_respecting_nesting(content);

        let mut values = Vec::new();
        for s in items {
            values.push(Value::infer(&s)?);
        }
        return Ok(Value::Tuple(values));
    }

    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let content = &trimmed[1..trimmed.len()-1];
        if content.trim().is_empty() { return Ok(Value::Vector(Vec::new())); }

        let items = split_respecting_nesting(content);
        let mut values = Vec::new();
        for s in items {
            values.push(Value::infer(&s)?);
        }
        return Ok(Value::Vector(values));
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let content = &trimmed[1..trimmed.len()-1];
        if content.trim().is_empty() { return Ok(Value::Vector(Vec::new())); }

        let items = split_respecting_nesting(content);
        if items.is_empty() { return Ok(Value::Vector(Vec::new())); }

        if contains_colon_at_top_level(&items[0]) {
            let mut map = HashMap::new();
            for item in items {
                if let Some((key_part, val_part)) = split_on_first_colon(&item) {
                    let key = key_part.trim();
                    let val_str = val_part.trim();

                    let key_clean = if key.starts_with('\'') && key.ends_with('\'') {
                        key[1..key.len()-1].to_string()
                    } else {
                        key.to_string()
                    };

                    map.insert(key_clean, Value::infer(val_str)?);
                }
            }
            return Ok(Value::HashMap(map));
        } else {
            let mut values = Vec::new();
            for s in items {
                values.push(Value::infer(&s)?);
            }
            return Ok(Value::Vector(values));
        }
    }

    Err(format!("Invalid complex type syntax: {}", trimmed))
}

pub fn contains_colon_at_top_level(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut state = ParseState::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let n1 = chars.get(i+1).cloned();
        let n2 = chars.get(i+2).cloned();

        let consumed = state.consume(c, n1, n2);

        if consumed == 1 && c == ':' && state.is_top_level() {
            return true;
        }
        i += consumed;
    }
    false
}

pub fn split_on_first_colon(s: &str) -> Option<(String, String)> {
    let chars: Vec<char> = s.chars().collect();
    let mut state = ParseState::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let n1 = chars.get(i+1).cloned();
        let n2 = chars.get(i+2).cloned();

        let consumed = state.consume(c, n1, n2);

        if consumed == 1 && c == ':' && state.is_top_level() {
            return Some((s[..i].to_string(), s[i+1..].to_string()));
        }
        i += consumed;
    }
    None
}

pub fn split_respecting_nesting(content: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut state = ParseState::new();

    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let n1 = chars.get(i+1).cloned();
        let n2 = chars.get(i+2).cloned();

        if c == ',' && state.is_top_level() {
            parts.push(current.trim().to_string());
            current.clear();
            i += 1;
            continue;
        }

        let consumed = state.consume(c, n1, n2);

        for offset in 0..consumed {
            current.push(chars[i + offset]);
        }

        i += consumed;
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}
