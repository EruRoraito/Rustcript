// File Version: 1.5.0
// /src/interpreter_step.rs

use crate::interpreter::Interpreter;
use crate::types::{Statement, ScriptHandler, PrintSegment};
use crate::data_types::Value;
use crate::operators;
use crate::flow_control;
use crate::loops;
use crate::stdlib;
use crate::functions;
use std::collections::HashMap;
use std::time::SystemTime;

pub fn execute<H: ScriptHandler>(
    interp: &mut Interpreter,
    handler: &mut H,
    pc: usize,
    stmt: &Statement
) -> Result<(bool, Option<usize>), String> {
    let mut jumped = false;
    let mut next_pc = None;

    match stmt {
        Statement::ModuleStart(name) => {
            interp.namespace_stack.push(name.clone());
        },
        Statement::ModuleEnd(name) => {
            if let Some(popped) = interp.namespace_stack.pop() {
                if popped != *name {
                    return Err(format!("Namespace integrity error: Expected to close '{}', but stack had '{}'", name, popped));
                }
            } else {
                return Err("Namespace stack underflow".to_string());
            }
        },
        Statement::Print(segments) => {
            let mut buf = String::new();
            for seg in segments {
                match seg {
                    PrintSegment::Literal(s) => buf.push_str(s),
                    PrintSegment::Variable(v) => {
                        let val = interp.resolve_val(v)?;
                        buf.push_str(&val.to_string());
                    }
                }
            }
            handler.on_print(&buf);
        },
        Statement::Input(var) => {
            let input_str = handler.on_input(var);
            interp.set_variable_auto(var.clone(), Value::parse_input(&input_str))?;
        },
        Statement::Time(target) => {
            interp.set_variable_auto(target.clone(), Value::Time(SystemTime::now()))?;
        },
        Statement::Exec { command, args } => {
             let raw_parts: Vec<&str> = args.split_whitespace().collect();
             let mut resolved = Vec::new();
             for part in raw_parts {
                 resolved.push(interp.resolve_val(part)?.to_string());
             }
             let final_args: Vec<&str> = resolved.iter().map(|s| s.as_str()).collect();
             if !handler.on_command(command, final_args)? {
                 return Err(format!("Unknown command: {}", command));
             }
        },
        Statement::MethodCall { target, object, method, args } => {
            let mut final_args = Vec::new();
            for arg in args {
                final_args.push(interp.resolve_val(arg)?);
            }

            if let Some(obj_val) = interp.get_var_mut(object) {
                let result = stdlib::call_method(obj_val, method, final_args)?;
                if let Some(tgt) = target {
                    interp.set_variable_auto(tgt.clone(), result.unwrap_or(Value::String("null".to_string())))?;
                }
            } else {
                let potential_label = format!("{}.{}", object, method);
                if let Some(&addr) = interp.program.labels.get(&potential_label) {
                     if let Some(Statement::FunctionDef { .. }) = interp.program.statements.get(addr) {
                        interp.enter_function_scope(&potential_label);
                        interp.arg_stack.push(final_args);
                        interp.call_stack.push(pc + 1);
                        interp.frames.push(HashMap::new());
                        interp.return_target_stack.push(target.clone());
                        next_pc = Some(addr);
                        jumped = true;
                     } else {
                         return Err(format!("Label '{}' exists but is not a function definition.", potential_label));
                     }
                } else {
                    let static_result = stdlib::call_static(
                        object,
                        method,
                        final_args.clone(),
                        interp.sandbox_root.as_deref(),
                        &interp.io_permissions
                    );
                    match static_result {
                        Ok(opt_val) => {
                            if let Some(tgt) = target {
                                interp.set_variable_auto(tgt.clone(), opt_val.unwrap_or(Value::String("null".to_string())))?;
                            }
                        },
                        Err(e) => return Err(e),
                    }
                }
            }
        },
        Statement::DefineGlobal { target, op, operand } => {
             let val = interp.resolve_val(operand)?;
             let res = operators::perform_assignment(&Value::Integer(0), op, &val)?;
             interp.set_variable_global(target.clone(), res);
        }
        Statement::DefineLocal { target, op, operand } => {
             let val = interp.resolve_val(operand)?;
             let res = operators::perform_assignment(&Value::Integer(0), op, &val)?;
             interp.set_variable_local(target.clone(), res);
        }
        Statement::CalcAssignment { target, op, operand } => {
            let current = interp.resolve_val(target).unwrap_or(Value::Integer(0));
            let val = interp.resolve_val(operand)?;
            let res = operators::perform_assignment(&current, op, &val)?;
            interp.set_variable_auto(target.clone(), res)?;
        }
        Statement::CalcArithmetic { target, left, op, right } => {
            let l = interp.resolve_val(left)?;
            let r = interp.resolve_val(right)?;
            let res = operators::perform_arithmetic(&l, op, &r)?;
            interp.set_variable_auto(target.clone(), res)?;
        }
        Statement::Call(label) => {
            let target_addr = if let Some(&addr) = interp.program.labels.get(label) {
                Some((addr, label.clone()))
            } else if !interp.namespace_stack.is_empty() {
                let ns_label = format!("{}.{}", interp.namespace_stack.join("."), label);
                interp.program.labels.get(&ns_label).map(|&a| (a, ns_label))
            } else {
                None
            };

            if let Some((addr, final_label)) = target_addr {
                interp.enter_function_scope(&final_label);
                interp.call_stack.push(pc + 1);
                interp.frames.push(HashMap::new());
                interp.return_target_stack.push(None);
                next_pc = Some(addr);
                jumped = true;
            } else {
                return Err(format!("Call unknown label: {}", label));
            }
        }
        Statement::FunctionDef { params, .. } => {
            if let Some(arg_values) = interp.arg_stack.pop() {
                 let locals = interp.frames.last_mut().expect("No stack frame for function");
                 functions::bind_arguments(locals, params, arg_values)?;
            } else {
                 if let Some(&end_idx) = interp.program.jump_map.get(&pc) {
                    next_pc = Some(end_idx + 1);
                    jumped = true;
                }
            }
        }
        Statement::EndFunction => {
            if let Some(addr) = interp.call_stack.pop() {
                interp.frames.pop();
                interp.return_target_stack.pop();
                interp.exit_function_scope()?;
                next_pc = Some(addr);
                jumped = true;
            } else {
                return Err("EndFunction found with empty stack".to_string());
            }
        }
        Statement::FunctionCall { target, name, args } => {
            let resolved_target = if let Some(&addr) = interp.program.labels.get(name) {
                 Some((addr, name.clone()))
            } else if !interp.namespace_stack.is_empty() {
                 let ns_name = format!("{}.{}", interp.namespace_stack.join("."), name);
                 interp.program.labels.get(&ns_name).map(|&a| (a, ns_name))
            } else {
                 match interp.resolve_val(name) {
                     Ok(Value::Function(label_name)) => {
                         if let Some(&addr) = interp.program.labels.get(&label_name) {
                             Some((addr, label_name))
                         } else {
                             return Err(format!("Runtime Error: Variable '{}' points to unknown function '{}'", name, label_name));
                         }
                     },
                     _ => None
                 }
            };

            if let Some((addr, final_name)) = resolved_target {
                if let Some(Statement::FunctionDef { .. }) = interp.program.statements.get(addr) {
                    let mut resolved_args = Vec::new();
                    for arg_expr in args {
                        resolved_args.push(interp.resolve_val(arg_expr)?);
                    }
                    interp.enter_function_scope(&final_name);
                    interp.arg_stack.push(resolved_args);
                    interp.call_stack.push(pc + 1);
                    interp.frames.push(HashMap::new());
                    interp.return_target_stack.push(target.clone());
                    next_pc = Some(addr);
                    jumped = true;
                } else {
                     return Err(format!("Target '{}' is a label but not a function definition.", final_name));
                }
            } else {
                if let Some(dot_idx) = name.rfind('.') {
                    let object_name = &name[..dot_idx];
                    let method_name = &name[dot_idx+1..];

                    let mut resolved_args = Vec::new();
                    for arg_expr in args {
                        resolved_args.push(interp.resolve_val(arg_expr)?);
                    }

                    if let Some(obj_val) = interp.get_var_mut(object_name) {
                        let result = stdlib::call_method(obj_val, method_name, resolved_args)?;
                        if let Some(tgt) = target {
                            interp.set_variable_auto(tgt.clone(), result.unwrap_or(Value::String("null".to_string())))?;
                        }
                    }
                    else {
                        match stdlib::call_static(object_name, method_name, resolved_args, interp.sandbox_root.as_deref(), &interp.io_permissions) {
                            Ok(opt_val) => {
                                if let Some(tgt) = target {
                                    interp.set_variable_auto(tgt.clone(), opt_val.unwrap_or(Value::String("null".to_string())))?;
                                }
                            },
                            Err(e) => {
                                return Err(format!("Unknown Function or Method: '{}'. (Error: {})", name, e));
                            }
                        }
                    }
                } else {
                    return Err(format!("Unknown Function: '{}'. (No label found, and not a method call)", name));
                }
            }
        }
        Statement::Return(val_expr) => {
            if let Some(addr) = interp.call_stack.pop() {
                let return_val = if let Some(expr) = val_expr {
                    Some(interp.resolve_val(expr)?)
                } else {
                    None
                };
                interp.frames.pop();
                interp.exit_function_scope()?;
                if let Some(target_opt) = interp.return_target_stack.pop() {
                     if let Some(target) = target_opt {
                         let val_to_set = return_val.unwrap_or(Value::Integer(0));
                         interp.set_variable_auto(target, val_to_set)?;
                     }
                }
                next_pc = Some(addr);
                jumped = true;
            } else {
                return Err("Return empty stack".to_string());
            }
        }

        Statement::If { .. } | Statement::ElseIf { .. } | Statement::Else |
        Statement::Goto(_) | Statement::Match { .. } | Statement::Break => {
            let locals = interp.frames.last().unwrap();
            let mut temp_pc = pc;

            flow_control::handle_branching(
                &mut temp_pc, stmt, &interp.program, &interp.globals, locals
            )?;

            if temp_pc != pc {
                next_pc = Some(temp_pc);
                jumped = true;
            }
        }
        Statement::Case { .. } | Statement::Default => {
            if let Some(&end_match) = interp.program.jump_map.get(&pc) {
                next_pc = Some(end_match);
                jumped = true;
            }
        }
        Statement::While { .. } | Statement::EndWhile |
        Statement::For { .. } | Statement::EndFor { .. } |
        Statement::Foreach { .. } | Statement::EndForeach { .. } |
        Statement::Loop => {
            let locals = interp.frames.last_mut().unwrap();
            let mut temp_pc = pc;
            loops::handle_loop(&mut temp_pc, stmt, &interp.program, &interp.globals, locals)?;
            if temp_pc != pc {
                next_pc = Some(temp_pc);
                jumped = true;
            }
        },

        Statement::Try => {
            if let Some(&catch_addr) = interp.program.jump_map.get(&pc) {
                interp.try_stack.push(catch_addr);
            } else {
                return Err("Try block missing Catch handler".to_string());
            }
        },
        Statement::EndTry => {
            interp.try_stack.pop();
            if let Some(&end_catch) = interp.program.jump_map.get(&pc) {
                next_pc = Some(end_catch);
                jumped = true;
            }
        },
        Statement::Catch => {},
        Statement::EndCatch => {},

        _ => {}
    }
    Ok((jumped, next_pc))
}
