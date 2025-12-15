// File Version: 3.8.0
// /src/types.rs

use std::collections::HashMap;

pub trait ScriptHandler {
    fn on_print(&mut self, text: &str);
    fn on_input(&mut self, variable_name: &str) -> String;
    fn on_command(&mut self, command: &str, args: Vec<&str>) -> Result<bool, String>;
}

#[derive(Debug, Clone, Copy)]
pub struct IoPermissions {
    pub read: bool,
    pub write: bool,
    pub delete: bool,
    pub allow_no_sandbox: bool,
}

impl Default for IoPermissions {
    fn default() -> Self {
        Self {
            read: false,
            write: false,
            delete: false,
            allow_no_sandbox: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PrintSegment {
    Literal(String),
    Variable(String),
}

pub struct Program {
    pub statements: Vec<Statement>,
    pub labels: HashMap<String, usize>,
    pub jump_map: HashMap<usize, usize>,
    pub debug_line_map: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Print(Vec<PrintSegment>),
    Input(String),

    Time(String),

    Exec { command: String, args: String },

    MethodCall {
        target: Option<String>,
        object: String,
        method: String,
        args: Vec<String>
    },

    CalcAssignment { target: String, op: String, operand: String },
    CalcArithmetic { target: String, left: String, op: String, right: String },
    DefineGlobal { target: String, op: String, operand: String },
    DefineLocal { target: String, op: String, operand: String },

    Label(String),
    Goto(String),

    Call(String),

    FunctionCall {
        target: Option<String>,
        name: String,
        args: Vec<String>
    },
    FunctionDef {
        name: String,
        params: Vec<String>,
    },
    EndFunction,

    Return(Option<String>),

    If { condition_parts: Vec<String> },
    ElseIf { condition_parts: Vec<String> },
    Else,
    EndIf,

    Match { var_name: String },
    Case { value: String },
    Default,
    EndMatch,

    Loop,
    While { condition_parts: Vec<String> },
    EndWhile,

    For { var: String, start: String, end: String },
    EndFor { var: String },

    Foreach { var: String, collection: String },
    EndForeach { var: String },

    Try,
    Catch,
    EndTry,
    EndCatch,

    ModuleStart(String),
    ModuleEnd(String),

    Break,
}
