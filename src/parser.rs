// # File Version: 5.7.2
// # /src/parser.rs

use crate::types::{Program, Statement, PrintSegment};
use crate::functions;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum BlockType {
    If,
    Else,
    While,
    For(String),
    Foreach(String),
    Match,
    Loop,
    Case,
    Try,
    Catch,
    Function,
    Module(String),
}

fn merge_multiline_lines(source: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    let mut buffer = String::new();
    let mut in_multiline = false;
    let mut start_line = 0;

    for (i, line) in source.lines().enumerate() {
        let current_line_num = i + 1;

        if in_multiline {
            buffer.push('\n');
            buffer.push_str(line);
            if line.trim().ends_with("'''") {
                result.push((start_line, buffer.clone()));
                buffer.clear();
                in_multiline = false;
            }
        } else {
            if let Some(idx) = line.find("'''") {
                let after_marker = &line[idx+3..];
                if after_marker.trim().is_empty() {
                    in_multiline = true;
                    start_line = current_line_num;
                    buffer.push_str(line);
                } else {
                    result.push((current_line_num, line.to_string()));
                }
            } else {
                result.push((current_line_num, line.to_string()));
            }
        }
    }
    if !buffer.is_empty() {
        result.push((start_line, buffer));
    }
    result
}

pub fn split_args(content: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth_parens = 0;
    let mut depth_braces = 0;
    let mut in_quote = false;

    for c in content.chars() {
        if c == '\'' {
            in_quote = !in_quote;
            current.push(c);
        } else if !in_quote {
            match c {
                '(' => { depth_parens += 1; current.push(c); }
                ')' => { if depth_parens > 0 { depth_parens -= 1; } current.push(c); }
                '{' => { depth_braces += 1; current.push(c); }
                '}' => { if depth_braces > 0 { depth_braces -= 1; } current.push(c); }
                ',' if depth_parens == 0 && depth_braces == 0 => {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        } else {
            current.push(c);
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn get_active_namespace(stack: &[(usize, BlockType)]) -> Option<String> {
    let mut parts = Vec::new();
    for (_, b_type) in stack {
        if let BlockType::Module(name) = b_type {
            parts.push(name.clone());
        }
    }
    if parts.is_empty() { None } else { Some(parts.join(".")) }
}

pub fn parse_source(source: &str) -> Result<Program, String> {
    let mut statements = Vec::new();
    let mut debug_lines = Vec::new();
    let mut labels = HashMap::new();
    let mut block_stack: Vec<(usize, BlockType)> = Vec::new();
    let mut jump_map = HashMap::new();
    let mut match_stack: Vec<Vec<usize>> = Vec::new();

    let lines = merge_multiline_lines(source);

    for (line_num, line) in lines {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed.is_empty() { continue; }

        let (stmt_str, is_block_end) = if trimmed.starts_with(']') {
            (trimmed[1..].trim(), true)
        } else {
            (trimmed, false)
        };

        if is_block_end {
            handle_block_close(
                line_num,
                &mut block_stack,
                &mut statements,
                &mut debug_lines,
                &mut jump_map,
                &mut match_stack
            )?;
        }

        if stmt_str.is_empty() { continue; }

        let (clean_stmt_str, is_block_start) = if stmt_str.ends_with('[') {
            (stmt_str[..stmt_str.len()-1].trim(), true)
        } else {
            (stmt_str, false)
        };

        if clean_stmt_str.is_empty() { continue; }

        let mut stmt = parse_line(clean_stmt_str)
            .map_err(|e| format!("Line {}: {}", line_num, e))?;

        if let Some(ns) = get_active_namespace(&block_stack) {
            match &mut stmt {
                Statement::Label(name) | Statement::FunctionDef { name, .. } => {
                    *name = format!("{}.{}", ns, name);
                },
                _ => {}
            }
        }

        if let Statement::Label(name) = &stmt {
            if labels.contains_key(name) { return Err(format!("Duplicate label '{}'", name)); }
            labels.insert(name.clone(), statements.len());
        }
        if let Statement::FunctionDef { name, .. } = &stmt {
            if labels.contains_key(name) { return Err(format!("Duplicate function/label name '{}'", name)); }
            labels.insert(name.clone(), statements.len());
        }

        let current_idx = statements.len();

        link_control_flow(
            line_num,
            &stmt,
            current_idx,
            &statements,
            &mut jump_map,
            &mut match_stack
        )?;

        statements.push(stmt.clone());
        debug_lines.push(line_num);

        if is_block_start {
            push_block_stack(line_num, &stmt, current_idx, &mut block_stack, &mut match_stack)?;
        }
    }

    if !block_stack.is_empty() {
        return Err("Unclosed block detected (missing ']')".to_string());
    }

    Ok(Program { statements, labels, jump_map, debug_line_map: debug_lines })
}

fn handle_block_close(
    line_num: usize,
    block_stack: &mut Vec<(usize, BlockType)>,
    statements: &mut Vec<Statement>,
    debug_lines: &mut Vec<usize>,
    jump_map: &mut HashMap<usize, usize>,
    match_stack: &mut Vec<Vec<usize>>
) -> Result<(), String> {
    let (start_idx, block_type) = block_stack.pop()
        .ok_or_else(|| format!("Line {}: Unexpected ']' (no block to close)", line_num))?;

    let current_idx = statements.len();

    match &block_type {
        BlockType::While | BlockType::Loop | BlockType::For(_) | BlockType::Foreach(_) => {
            jump_map.insert(start_idx, current_idx);
            jump_map.insert(current_idx, start_idx);
        },
        BlockType::If | BlockType::Else | BlockType::Try | BlockType::Catch | BlockType::Match | BlockType::Function => {
            jump_map.insert(start_idx, current_idx);
        },
        BlockType::Module(_) | BlockType::Case => {},
    }

    let closing_stmt = match block_type {
        BlockType::While => Statement::EndWhile,
        BlockType::Loop => Statement::EndWhile,
        BlockType::For(var) => Statement::EndFor { var },
        BlockType::Foreach(var) => Statement::EndForeach { var },
        BlockType::If | BlockType::Else => Statement::EndIf,
        BlockType::Match => {
            if let Some(cases) = match_stack.pop() {
                for case_idx in cases {
                    jump_map.insert(case_idx, current_idx);
                }
            } else {
                 return Err(format!("Line {}: Internal Match Stack Error", line_num));
            }
            Statement::EndMatch
        },
        BlockType::Case => return Ok(()),
        BlockType::Try => Statement::EndTry,
        BlockType::Catch => Statement::EndCatch,
        BlockType::Function => Statement::EndFunction,
        BlockType::Module(name) => Statement::ModuleEnd(name),
    };

    statements.push(closing_stmt);
    debug_lines.push(line_num);

    Ok(())
}

fn push_block_stack(
    line_num: usize,
    stmt: &Statement,
    current_idx: usize,
    block_stack: &mut Vec<(usize, BlockType)>,
    match_stack: &mut Vec<Vec<usize>>
) -> Result<(), String> {
    let b_type = match stmt {
        Statement::If { .. } => BlockType::If,
        Statement::Else | Statement::ElseIf { .. } => BlockType::Else,
        Statement::While { .. } => BlockType::While,
        Statement::For { var, .. } => BlockType::For(var.clone()),
        Statement::Foreach { var, .. } => BlockType::Foreach(var.clone()),
        Statement::Loop => BlockType::Loop,
        Statement::Match { .. } => {
            match_stack.push(Vec::new());
            BlockType::Match
        },
        Statement::Case{..} | Statement::Default => BlockType::Case,
        Statement::Try => BlockType::Try,
        Statement::Catch => BlockType::Catch,
        Statement::FunctionDef { .. } => BlockType::Function,
        Statement::ModuleStart(name) => BlockType::Module(name.clone()),
        _ => return Err(format!("Line {}: This command cannot start a block", line_num)),
    };
    block_stack.push((current_idx, b_type));
    Ok(())
}

fn link_control_flow(
    line_num: usize,
    stmt: &Statement,
    current_idx: usize,
    statements: &Vec<Statement>,
    jump_map: &mut HashMap<usize, usize>,
    match_stack: &mut Vec<Vec<usize>>
) -> Result<(), String> {
    if matches!(stmt, Statement::Else | Statement::ElseIf {..}) {
        if let Some(Statement::EndIf) = statements.last() {
            let prev_endif_idx = current_idx - 1;
            let found_prev = jump_map.iter()
                .find(|(_, &dest)| dest == prev_endif_idx)
                .map(|(&src, _)| src);

            if let Some(prev_start) = found_prev {
                let target = if matches!(stmt, Statement::Else) { current_idx + 1 } else { current_idx };
                jump_map.insert(prev_start, target);
            } else {
               return Err(format!("Line {}: 'else' linkage failed.", line_num));
            }
        } else {
             return Err(format!("Line {}: 'else' must follow ']' (EndIf)", line_num));
        }
    }

    if matches!(stmt, Statement::Catch) {
        if let Some(Statement::EndTry) = statements.last() {
            let end_try_idx = current_idx - 1;
            let try_start_idx = jump_map.iter()
                .find(|(_, &dest)| dest == end_try_idx)
                .map(|(&src, _)| src);

            if let Some(try_idx) = try_start_idx {
                jump_map.insert(try_idx, current_idx);
            } else {
                return Err(format!("Line {}: 'catch' linkage failed", line_num));
            }
        } else {
            return Err(format!("Line {}: 'catch' must immediately follow 'try [...]'", line_num));
        }
    }

    if matches!(stmt, Statement::Case{..} | Statement::Default) {
         if let Some(cases) = match_stack.last_mut() {
             cases.push(current_idx);
         } else {
             return Err(format!("Line {}: Case/Default outside of Match", line_num));
         }
    }
    Ok(())
}

fn parse_line(line: &str) -> Result<Statement, String> {
    let trimmed = line.trim();

    let (cmd, rest) = if let Some(idx) = trimmed.find(char::is_whitespace) {
        (trimmed[..idx].trim(), trimmed[idx..].trim())
    } else {
        (trimmed, "")
    };

    match cmd {
        "print" => return parse_template(rest).map(Statement::Print),
        "input" => return Ok(Statement::Input(strip_legacy_assign(rest).to_string())),
        "time" => return Ok(Statement::Time(strip_legacy_assign(rest).to_string())),
        "method" => return parse_method(rest),
        "goto" => return Ok(Statement::Goto(strip_legacy_assign(rest).to_string())),
        "label" => return Ok(Statement::Label(strip_legacy_assign(rest).to_string())),
        "function" => return functions::parse_definition(rest).map(|(name, params)| Statement::FunctionDef { name, params }),
        "module" => return Ok(Statement::ModuleStart(strip_legacy_assign(rest).to_string())),
        "exec" => return parse_exec(rest),
        "if" => return Ok(Statement::If { condition_parts: split_condition(rest) }),
        "else_if" => return Ok(Statement::ElseIf { condition_parts: split_condition(rest) }),
        "match" => return Ok(Statement::Match { var_name: strip_legacy_assign(rest).to_string() }),
        "case" => return Ok(Statement::Case { value: strip_legacy_assign(rest).to_string() }),
        "while" => return Ok(Statement::While { condition_parts: split_condition(rest) }),
        "for" => return parse_for(rest),
        "foreach" => return parse_foreach(rest),
        "call" => return Ok(Statement::Call(strip_legacy_assign(rest).to_string())),
        "return" => {
            let val = strip_legacy_assign(rest);
            return Ok(Statement::Return(if val.is_empty() { None } else { Some(val.to_string()) }));
        },
        "else" => return Ok(Statement::Else),
        "loop" => return Ok(Statement::Loop),
        "break" => return Ok(Statement::Break),
        "default" => return Ok(Statement::Default),
        "try" => return Ok(Statement::Try),
        "catch" => return Ok(Statement::Catch),
        "global" => return parse_assignment_or_arithmetic(rest, true, false),
        "var" | "local" => return parse_assignment_or_arithmetic(rest, false, true),
        _ => {}
    }

    parse_assignment_or_arithmetic(trimmed, false, false)
}

fn strip_legacy_assign(raw: &str) -> &str {
    let s = raw.trim();
    if s.starts_with('=') {
        s[1..].trim()
    } else {
        s
    }
}

fn split_condition(rest: &str) -> Vec<String> {
    let clean = strip_legacy_assign(rest);
    clean.split_whitespace().map(String::from).collect()
}

fn parse_exec(value: &str) -> Result<Statement, String> {
    let trimmed_val = strip_legacy_assign(value);
    if let Some(space_idx) = trimmed_val.find(' ') {
        let (cmd_part, args_part) = trimmed_val.split_at(space_idx);
        Ok(Statement::Exec { command: cmd_part.to_string(), args: args_part.trim().to_string() })
    } else {
        Ok(Statement::Exec { command: trimmed_val.to_string(), args: String::new() })
    }
}

fn parse_for(value: &str) -> Result<Statement, String> {
    let clean = strip_legacy_assign(value);
    let p: Vec<String> = clean.split_whitespace().map(String::from).collect();
    if p.len() != 3 { return Err("Invalid for loop format. Expected 'var start end'".to_string()); }
    Ok(Statement::For { var: p[0].clone(), start: p[1].clone(), end: p[2].clone() })
}

fn parse_foreach(value: &str) -> Result<Statement, String> {
    let clean = strip_legacy_assign(value);
    let p: Vec<String> = clean.split_whitespace().map(String::from).collect();
    if p.len() != 3 || p[1] != "in" { return Err("Invalid foreach format. Expected 'var in collection'".to_string()); }
    Ok(Statement::Foreach { var: p[0].clone(), collection: p[2].clone() })
}

fn parse_method(value: &str) -> Result<Statement, String> {
    let inner = strip_legacy_assign(value);
    let (target, rest) = if let Some(idx) = inner.find('=') {
        let t = inner[..idx].trim().to_string();
        let r = inner[idx+1..].trim();
        (Some(t), r)
    } else {
        (None, inner)
    };
    let dot_idx = rest.find('.').ok_or("Method call requires object.method()")?;
    let object = rest[..dot_idx].trim().to_string();
    let after_dot = &rest[dot_idx+1..];
    let paren_idx = after_dot.find('(').ok_or("Method call requires (...)")?;
    let method = after_dot[..paren_idx].trim().to_string();
    if !after_dot.ends_with(')') { return Err("Missing closing ')'".to_string()); }

    let args_str = &after_dot[paren_idx+1 .. after_dot.len()-1];
    let args = if args_str.trim().is_empty() { Vec::new() } else { split_args(args_str) };

    Ok(Statement::MethodCall { target, object, method, args })
}

fn parse_assignment_or_arithmetic(line: &str, is_global: bool, is_local: bool) -> Result<Statement, String> {
    let has_paren = line.contains('(') && line.ends_with(')');

    if has_paren {
        if let Ok((target, name, args)) = functions::parse_call(line) {
             return Ok(Statement::FunctionCall { target, name, args });
        }
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() { return Err("Invalid expression".to_string()); }

    if parts.len() >= 2 && parts[1] == "=" {
        let target = parts[0].to_string();
        let eq_idx = line.find('=').unwrap();
        let operand = line[eq_idx+1..].trim().to_string();

        if is_global {
            return Ok(Statement::DefineGlobal { target, op: "=".to_string(), operand });
        } else if is_local {
            return Ok(Statement::DefineLocal { target, op: "=".to_string(), operand });
        } else {
            return Ok(Statement::CalcAssignment { target, op: "=".to_string(), operand });
        }
    }

    let assignment_ops = ["+=", "-=", "*=", "/=", "%="];
    if parts.len() >= 2 && assignment_ops.contains(&parts[1]) {
         let target = parts[0].to_string();
         let op = parts[1].to_string();
         let op_idx = line.find(&op).unwrap();
         let operand = line[op_idx+op.len()..].trim().to_string();

         if is_global || is_local {
             return Err("Compound assignment (+=, -=) not supported in variable declaration. Use 'var x = 1' then 'x += 1'.".to_string());
         }
         return Ok(Statement::CalcAssignment { target, op, operand });
    }

    if parts.len() >= 4 {

        let target = parts[0].to_string();
        let left = parts[1].to_string();
        let op = parts[2].to_string();

        if ["+", "-", "*", "/", "%", "==", "!=", ">", "<", ">=", "<=", "&&", "||"].contains(&op.as_str()) {
             let op_idx = line.find(&op).unwrap();

             let after_op_start = op_idx + op.len();

             let right = line[after_op_start..].trim().to_string();
             return Ok(Statement::CalcArithmetic { target, left, op, right });
        }
    }

    Err(format!("Unrecognized assignment or arithmetic expression: '{}'", line))
}

fn parse_template(template: &str) -> Result<Vec<PrintSegment>, String> {
    let trimmed = strip_legacy_assign(template);
    let is_triple = trimmed.starts_with("'''") && trimmed.ends_with("'''") && trimmed.len() >= 6;
    let is_single = trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2;

    if !is_triple && !is_single {
        return Ok(vec![PrintSegment::Variable(trimmed.to_string())]);
    }

    let content = if is_triple {
        &trimmed[3..trimmed.len() - 3]
    } else {
        &trimmed[1..trimmed.len() - 1]
    };

    let mut segments = Vec::new();
    let mut last_pos = 0;

    for (start_pos, _) in content.match_indices('{') {
        if start_pos > last_pos {
            segments.push(PrintSegment::Literal(content[last_pos..start_pos].to_string()));
        }

        if let Some(offset) = content[start_pos..].find('}') {
            let end_pos = start_pos + offset;
            let var = content[start_pos + 1..end_pos].to_string();
            segments.push(PrintSegment::Variable(var));
            last_pos = end_pos + 1;
        } else {
            return Err("Mismatched braces in print template".to_string());
        }
    }

    if last_pos < content.len() {
        segments.push(PrintSegment::Literal(content[last_pos..].to_string()));
    }
    Ok(segments)
}
