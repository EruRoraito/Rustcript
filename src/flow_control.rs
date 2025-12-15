// File Version: 2.5.0
// /src/flow_control.rs

use crate::types::{Program, Statement};
use crate::data_types::Value;
use crate::operators;
use crate::match_control;
use std::collections::HashMap;

fn resolve(token: &str, globals: &HashMap<String, Value>, locals: &HashMap<String, Value>) -> Result<Value, String> {
    if let Some(val) = locals.get(token) {
        return Ok(val.clone());
    }
    if let Some(val) = globals.get(token) {
        return Ok(val.clone());
    }
    match Value::infer(token) {
        Ok(v) => Ok(v),
        Err(e) => {
             if token.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '-') {
                 Err(e)
             } else {
                 Err(format!("Variable '{}' not found (and not a quoted string).", token))
             }
        }
    }
}

fn is_true(parts: &[String], globals: &HashMap<String, Value>, locals: &HashMap<String, Value>) -> Result<bool, String> {
    if parts.len() == 1 {
        let val = resolve(&parts[0], globals, locals)?;
        return Ok(val.as_bool());
    }

    if parts.len() == 2 && parts[0] == "!" {
         let val = resolve(&parts[1], globals, locals)?;
         return operators::perform_unary_logic("!", &val);
    }

    if parts.len() == 3 {
        let left = resolve(&parts[0], globals, locals)?;
        let right = resolve(&parts[2], globals, locals)?;
        let op = &parts[1];

        if op == "&&" || op == "||" {
            return operators::perform_logic(&left, op, &right);
        }

        return operators::perform_comparison(&left, op, &right);
    }

    Err(format!("Invalid Condition format: {:?}", parts))
}

pub fn handle_branching(
    pc: &mut usize,
    stmt: &Statement,
    program: &Program,
    globals: &HashMap<String, Value>,
    locals: &HashMap<String, Value>
) -> Result<(), String> {
    match stmt {
        Statement::If { condition_parts } => {
            if condition_parts.len() == 4 {
                let cond_slice = &condition_parts[0..3];
                let dest_label = &condition_parts[3];
                if is_true(cond_slice, globals, locals)? {
                    if let Some(&addr) = program.labels.get(dest_label) {
                        *pc = addr;
                        return Ok(());
                    } else {
                        return Err(format!("Legacy If-Goto unknown label: {}", dest_label));
                    }
                }
                return Ok(());
            }

            let result = is_true(condition_parts, globals, locals)?;
            if !result {
                if let Some(&dest) = program.jump_map.get(pc) {
                    *pc = dest;
                    return Ok(());
                } else {
                    return Err("If block missing jump target".to_string());
                }
            }
        }
        Statement::ElseIf { condition_parts } => {
            let result = is_true(condition_parts, globals, locals)?;
            if !result {
                if let Some(&dest) = program.jump_map.get(pc) {
                    *pc = dest;
                    return Ok(());
                } else {
                    return Err("ElseIf missing jump target".to_string());
                }
            }
        }
        Statement::Else => {
             if let Some(&dest) = program.jump_map.get(pc) {
                *pc = dest;
            }
        }
        Statement::Goto(label) => {
            if let Some(&addr) = program.labels.get(label) {
                *pc = addr;
            } else {
                return Err(format!("Goto unknown label: {}", label));
            }
        }
        Statement::Match { var_name } => {
            match_control::execute(pc, var_name, program, globals, locals)?;
        }
        Statement::Break => {
             if let Some(&dest) = program.jump_map.get(pc) {
                 *pc = dest;
             } else {
                 return Err("Break used outside of supported block".to_string());
             }
        }
        _ => {}
    }
    Ok(())
}
