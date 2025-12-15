// File Version: 13.6.0
// /src/interpreter.rs

use crate::types::{Program, ScriptHandler, IoPermissions};
use crate::data_types::Value;
use crate::parser;
use crate::complex_types;
use crate::interpreter_utils::{self, AccessOp};
use crate::interpreter_step;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Interpreter {
    pub(crate) program: Program,
    pub(crate) globals: HashMap<String, Value>,
    pub(crate) frames: Vec<HashMap<String, Value>>,

    pub(crate) call_stack: Vec<usize>,
    pub(crate) try_stack: Vec<usize>,
    pub(crate) return_target_stack: Vec<Option<String>>,
    pub(crate) arg_stack: Vec<Vec<Value>>,
    pub(crate) namespace_stack: Vec<String>,
    pub(crate) namespace_backup_stack: Vec<Vec<String>>,

    pub(crate) instruction_count: usize,
    pub(crate) max_instructions: usize,
    pub(crate) sandbox_root: Option<PathBuf>,
    pub(crate) io_permissions: IoPermissions,
}

impl Interpreter {
    pub fn from_source(source: &str) -> Result<Self, String> {
        let program = parser::parse_source(source)?;
        Ok(Self {
            program,
            globals: HashMap::new(),
            frames: vec![HashMap::new()],
            call_stack: Vec::new(),
            try_stack: Vec::new(),
            return_target_stack: Vec::new(),
            arg_stack: Vec::new(),
            namespace_stack: Vec::new(),
            namespace_backup_stack: Vec::new(),
            instruction_count: 0,
            max_instructions: 0,
            sandbox_root: None,
            io_permissions: IoPermissions::default(),
        })
    }

    pub fn set_instruction_limit(&mut self, limit: usize) {
        self.max_instructions = limit;
    }

    pub fn set_sandbox_root(&mut self, path: PathBuf) {
        self.sandbox_root = Some(path);
    }

    pub fn set_io_permissions(&mut self, perms: IoPermissions) {
        self.io_permissions = perms;
    }

    pub fn set_global(&mut self, name: &str, value: Value) {
        self.set_variable_global(name.to_string(), value);
    }

    pub fn get_value(&self, name: &str) -> Option<Value> {
        if let Some(frame) = self.frames.last() {
            if let Some(val) = frame.get(name) {
                return Some(val.clone());
            }
        }
        self.globals.get(name).cloned()
    }

    pub fn run<H: ScriptHandler>(&mut self, handler: &mut H) -> Result<(), String> {
        let mut pc = 0;

        while pc < self.program.statements.len() {
            if self.max_instructions > 0 {
                self.instruction_count += 1;
                if self.instruction_count > self.max_instructions {
                    return Err(format!("Execution Limit Exceeded: Stopped after {} instructions.", self.max_instructions));
                }
            }

            let stmt = self.program.statements[pc].clone();

            match interpreter_step::execute(self, handler, pc, &stmt) {
                Ok((jumped, next)) => {
                    pc = if jumped { next.unwrap() } else { pc + 1 };
                },
                Err(e) => {
                    let line_num = self.program.debug_line_map.get(pc).unwrap_or(&0);
                    let detailed_err = format!("Error [Line {}]: {}", line_num, e);

                    if let Some(catch_pc) = self.try_stack.pop() {
                        self.set_variable_global("LAST_ERROR".to_string(), Value::String(detailed_err));
                        pc = catch_pc;
                    } else {
                        return Err(detailed_err);
                    }
                }
            }
        }
        Ok(())
    }

    fn get_namespaced_key(&self, name: &str) -> Option<String> {
        if self.namespace_stack.is_empty() {
            None
        } else {
            Some(format!("{}.{}", self.namespace_stack.join("."), name))
        }
    }

    pub(crate) fn get_var_mut<'a>(&'a mut self, name: &str) -> Option<&'a mut Value> {
        let ns_key = self.get_namespaced_key(name);

        if let Some(frame) = self.frames.last_mut() {
            if frame.contains_key(name) {
                return frame.get_mut(name);
            }
        }

        if self.globals.contains_key(name) {
            return self.globals.get_mut(name);
        }

        if let Some(key) = ns_key {
            if self.globals.contains_key(&key) {
                return self.globals.get_mut(&key);
            }
        }
        None
    }

    pub(crate) fn set_variable_local(&mut self, name: String, value: Value) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name, value);
        }
    }

    pub(crate) fn set_variable_global(&mut self, name: String, value: Value) {
        let key = self.get_namespaced_key(&name).unwrap_or(name);
        self.globals.insert(key, value);
    }

    pub(crate) fn enter_function_scope(&mut self, func_name: &str) {
        self.namespace_backup_stack.push(self.namespace_stack.clone());

        if let Some(dot_idx) = func_name.rfind('.') {
            let ns_path = &func_name[..dot_idx];
            self.namespace_stack = ns_path.split('.').map(String::from).collect();
        } else {
            self.namespace_stack.clear();
        }
    }

    pub(crate) fn exit_function_scope(&mut self) -> Result<(), String> {
        self.namespace_stack = self.namespace_backup_stack.pop()
            .ok_or_else(|| "Stack underflow: Attempted to exit function scope without backup".to_string())?;
        Ok(())
    }

    fn resolve_basic_var(&self, token: &str) -> Option<Value> {
        if let Some(frame) = self.frames.last() {
            if let Some(val) = frame.get(token) { return Some(val.clone()); }
        }
        if let Some(val) = self.globals.get(token) { return Some(val.clone()); }

        if let Some(ns_key) = self.get_namespaced_key(token) {
             if let Some(val) = self.globals.get(&ns_key) { return Some(val.clone()); }
        }
        None
    }

    pub fn resolve_val(&self, token: &str) -> Result<Value, String> {
        let trimmed = token.trim();

        if trimmed.starts_with('\'') {
             return Value::infer(trimmed);
        }
        if trimmed.starts_with('{') || trimmed.starts_with('(') || trimmed.starts_with('[') {
            return self.resolve_complex_structure(trimmed);
        }

        if let Some(val) = self.resolve_basic_var(trimmed) {
            return Ok(val);
        }


        let first_char = trimmed.chars().next().unwrap_or(' ');
        let is_identifier_start = first_char.is_ascii_alphabetic() || first_char == '_';

        let looks_like_chain = (trimmed.contains('.') || trimmed.contains('[')) && is_identifier_start;

        if looks_like_chain {
            let (root, ops) = interpreter_utils::parse_access_chain(trimmed);

            if !ops.is_empty() {
                let mut current = self.resolve_basic_var(&root)
                    .or_else(|| Value::infer(&root).ok())
                    .ok_or_else(|| format!("Variable '{}' not found", root))?;

                for op in ops {
                    match op {
                        AccessOp::Dot(prop) => {
                             current = interpreter_utils::access_property(&current, &prop)
                                .ok_or_else(|| format!("Property '{}' not found on {}", prop, root))?;
                        },
                        AccessOp::Bracket(expr) => {
                            let index_val = self.resolve_val(&expr)?;
                            current = interpreter_utils::access_dynamic(&current, index_val)
                                .ok_or_else(|| format!("Index access failed on {}", root))?;
                        }
                    }
                }
                return Ok(current);
            }
        }

        match Value::infer(trimmed) {
            Ok(v) => Ok(v),
            Err(e) => {
                 if self.program.labels.contains_key(trimmed) {
                     return Ok(Value::Function(trimmed.to_string()));
                 }
                 if let Some(ns_key) = self.get_namespaced_key(trimmed) {
                     if self.program.labels.contains_key(&ns_key) {
                         return Ok(Value::Function(ns_key));
                     }
                 }

                 if trimmed.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '-') {
                     Err(e)
                 } else {
                     Err(format!("Variable or Function '{}' not found.", trimmed))
                 }
            }
        }
    }

    fn resolve_complex_structure(&self, raw: &str) -> Result<Value, String> {
        let trimmed = raw.trim();

        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let content = &trimmed[1..trimmed.len()-1];
            let items = complex_types::split_respecting_nesting(content);
            let values = items.into_iter().map(|item| self.resolve_val(&item)).collect::<Result<_,_>>()?;
            return Ok(Value::Tuple(values));
        }

        let is_brace = trimmed.starts_with('{') && trimmed.ends_with('}');
        let is_bracket = trimmed.starts_with('[') && trimmed.ends_with(']');

        if is_brace || is_bracket {
             let content = &trimmed[1..trimmed.len()-1];
             if content.trim().is_empty() { return Ok(Value::Vector(Vec::new())); }

             let items = complex_types::split_respecting_nesting(content);
             if items.is_empty() { return Ok(Value::Vector(Vec::new())); }

             let first = &items[0];

             if is_brace && complex_types::contains_colon_at_top_level(first) {
                 let mut map = HashMap::new();
                 for item in items {
                     if let Some((key_part, val_part)) = complex_types::split_on_first_colon(&item) {
                         let key = key_part.trim();
                         let key_clean = if key.starts_with('\'') && key.ends_with('\'') {
                             key[1..key.len()-1].to_string()
                         } else {
                             key.to_string()
                         };
                         let val = self.resolve_val(val_part.trim())?;
                         map.insert(key_clean, val);
                     } else {
                         return Err(format!("Invalid map item: {}", item));
                     }
                 }
                 return Ok(Value::HashMap(map));
             } else {
                 let values = items.into_iter().map(|item| self.resolve_val(&item)).collect::<Result<_,_>>()?;
                 return Ok(Value::Vector(values));
             }
        }
        Err("Not a valid complex structure".to_string())
    }

    pub(crate) fn set_variable_auto(&mut self, name: String, value: Value) -> Result<(), String> {

        if let Some(frame) = self.frames.last_mut() {
            if frame.contains_key(&name) {
                frame.insert(name, value);
                return Ok(());
            }
        }

        if self.globals.contains_key(&name) {
            self.globals.insert(name, value);
            return Ok(());
        }

        if name.contains('.') || name.contains('[') {
             let is_numeric = name.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E');

             if !is_numeric {
                 let (root, ops) = interpreter_utils::parse_access_chain(&name);

                 if !ops.is_empty() {
                     let mut resolved_keys = Vec::new();
                     for op in ops {
                         match op {
                             AccessOp::Dot(s) => resolved_keys.push(Value::String(s)),
                             AccessOp::Bracket(expr) => resolved_keys.push(self.resolve_val(&expr)?),
                         }
                     }

                     if let Some(root_var) = self.get_var_mut(&root) {
                         interpreter_utils::mutate_chain(root_var, resolved_keys, value)?;
                         return Ok(());
                     }
                 }
             }
        }

        if let Some(ns_key) = self.get_namespaced_key(&name) {
            if self.globals.contains_key(&ns_key) {
                self.globals.insert(ns_key, value);
                return Ok(());
            }
        }

        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name, value);
        }
        Ok(())
    }
}
