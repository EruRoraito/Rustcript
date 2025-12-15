// File Version: 1.1.0
// /src/match_control.rs

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

pub fn execute(
    pc: &mut usize,
    var_name: &str,
    program: &Program,
    globals: &HashMap<String, Value>,
    locals: &HashMap<String, Value>
) -> Result<(), String> {
    let val = resolve(var_name, globals, locals)?;
    let mut default_pc: Option<usize> = None;
    let mut scan_pc = *pc + 1;

    while scan_pc < program.statements.len() {
        match &program.statements[scan_pc] {
            Statement::Case { value } => {
                let case_val = Value::infer(value)?;
                if operators::perform_comparison(&val, "==", &case_val)? {
                    *pc = scan_pc + 1;
                    return Ok(());
                }
            },
            Statement::Default => {
                default_pc = Some(scan_pc + 1);
            },
            Statement::EndMatch => {
                if let Some(def) = default_pc {
                    *pc = def;
                } else {
                    *pc = scan_pc;
                }
                return Ok(());
            }
            _ => {}
        }
        scan_pc += 1;
    }
    Ok(())
}
