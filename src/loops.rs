// File Version: 3.1.0
// /src/loops.rs

use crate::types::{Program, Statement};
use crate::data_types::Value;
use crate::operators;
use std::collections::HashMap;

fn resolve(token: &str, globals: &HashMap<String, Value>, locals: &HashMap<String, Value>) -> Result<Value, String> {
    if let Some(val) = locals.get(token) {
        return Ok(val.clone());
    }
    if let Some(val) = globals.get(token) {
        return Ok(val.clone());
    }
    Value::infer(token)
}

pub fn handle_loop(
    pc: &mut usize,
    stmt: &Statement,
    program: &Program,
    globals: &HashMap<String, Value>,
    locals: &mut HashMap<String, Value>
) -> Result<(), String> {

    match stmt {
        Statement::While { condition_parts } => {
             let res = if condition_parts.len() == 1 {
                 resolve(&condition_parts[0], globals, locals)?.as_bool()
             } else if condition_parts.len() == 2 && condition_parts[0] == "!" {
                 !resolve(&condition_parts[1], globals, locals)?.as_bool()
             } else if condition_parts.len() == 3 {
                 let left = resolve(&condition_parts[0], globals, locals)?;
                 let right = resolve(&condition_parts[2], globals, locals)?;
                 let op = &condition_parts[1];

                 if op == "&&" || op == "||" {
                     operators::perform_logic(&left, op, &right)?
                 } else {
                     operators::perform_comparison(&left, op, &right)?
                 }
             } else {
                 return Err("Invalid While condition format".to_string());
             };

             if !res {
                 if let Some(&end) = program.jump_map.get(pc) {
                     *pc = end + 1;
                 }
             }
        }
        Statement::EndWhile | Statement::EndFor { .. } => {
             if let Some(&start) = program.jump_map.get(pc) {
                 *pc = start;
                 if let Statement::EndFor { var } = stmt {
                     let mut current = locals.get(var).cloned();
                     if current.is_none() { current = globals.get(var).cloned(); }
                     let val = current.unwrap_or(Value::Integer(0));
                     let incremented = operators::perform_arithmetic(&val, "+", &Value::Integer(1))?;
                     locals.insert(var.clone(), incremented);
                 }
             }
        }
        Statement::For { var, start, end } => {
             let start_val = Value::infer(start)?;
             let end_val = Value::infer(end)?;

             if !locals.contains_key(var) && !globals.contains_key(var) {
                 locals.insert(var.clone(), start_val.clone());
             }

             let mut current = locals.get(var).cloned();
             if current.is_none() { current = globals.get(var).cloned(); }
             let val = current.unwrap();

             if operators::perform_comparison(&val, ">=", &end_val)? {
                 if let Some(&end_idx) = program.jump_map.get(pc) {
                     *pc = end_idx + 1;
                 }
             }
        }
        Statement::Foreach { var, collection } => {
             let idx_var = format!("__idx_{}", var);
             let col_val = resolve(collection, globals, locals)?;

             if !locals.contains_key(&idx_var) {
                 locals.insert(idx_var.clone(), Value::Integer(0));

                 if let Value::HashMap(map) = &col_val {
                     let keys: Vec<Value> = map.keys().map(|k| Value::String(k.clone())).collect();
                     locals.insert(format!("__keys_{}", var), Value::Vector(keys));
                 }
             }

             let idx_val = locals.get(&idx_var).unwrap().as_float().unwrap() as usize;
             let mut finished = false;

             match &col_val {
                 Value::Vector(vec) | Value::Tuple(vec) => {
                     if idx_val < vec.len() {
                         locals.insert(var.clone(), vec[idx_val].clone());
                     } else {
                         finished = true;
                     }
                 },
                 Value::HashMap(_) => {
                     let keys_var = format!("__keys_{}", var);
                     if let Some(Value::Vector(keys)) = locals.get(&keys_var) {
                         if idx_val < keys.len() {
                             locals.insert(var.clone(), keys[idx_val].clone());
                         } else {
                             finished = true;
                         }
                     } else {
                         finished = true;
                     }
                 }
                 _ => return Err(format!("Cannot iterate over {}", col_val.type_name())),
             }

             if finished {
                 if let Some(&end_idx) = program.jump_map.get(pc) {
                     locals.remove(&idx_var);
                     locals.remove(&format!("__keys_{}", var));
                     *pc = end_idx + 1;
                 }
             }
        },
        Statement::EndForeach { var } => {
             let idx_var = format!("__idx_{}", var);
             if let Some(val) = locals.get(&idx_var) {
                 let next = operators::perform_arithmetic(val, "+", &Value::Integer(1))?;
                 locals.insert(idx_var, next);
             }
             if let Some(&start) = program.jump_map.get(pc) {
                 *pc = start;
             }
        }
        _ => {}
    }
    Ok(())
}
