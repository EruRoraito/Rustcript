//  File Version: 1.4.0
//  /src/importer.rs

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve(entry_file_path: &str) -> Result<String, String> {
    let mut visited = HashSet::new();
    let root_path = PathBuf::from(entry_file_path);

    if !root_path.exists() {
        return Err(format!("Entry file not found: {}", entry_file_path));
    }

    let canonical = fs::canonicalize(&root_path)
        .map_err(|e| format!("Error resolving path {}: {}", entry_file_path, e))?;

    resolve_recursive(&canonical, &mut visited)
}

fn resolve_recursive(current_path: &Path, visited: &mut HashSet<PathBuf>) -> Result<String, String> {
    if visited.contains(current_path) {
        return Ok(String::new());
    }
    visited.insert(current_path.to_path_buf());

    let content = fs::read_to_string(current_path)
        .map_err(|e| format!("Failed to read file {:?}: {}", current_path, e))?;

    let mut combined_source = String::new();
    let file_name = current_path.file_name().unwrap_or_default().to_string_lossy();

    combined_source.push_str(&format!("\n# --- BEGIN IMPORT: {} ---\n", file_name));

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.split('#').next().unwrap_or("").trim();

        let is_import = trimmed.starts_with("import ") || trimmed.starts_with("import=") || trimmed == "import";

        if is_import {
            let (rel_path, alias) = parse_import_line(trimmed, line_num + 1)?;

            let parent_dir = current_path.parent().unwrap_or_else(|| Path::new("."));
            let target_path = parent_dir.join(&rel_path);

            if !target_path.exists() {
                 return Err(format!("Import not found: '{}' in {:?}", rel_path, current_path));
            }

            let abs_target = fs::canonicalize(&target_path)
                .map_err(|e| format!("Path resolution error: {}", e))?;

            let imported_code = resolve_recursive(&abs_target, visited)?;

            if let Some(mod_name) = alias {
                combined_source.push_str(&format!("\nmodule {} [\n", mod_name));
                combined_source.push_str(&imported_code);
                combined_source.push_str("\n]\n");
            } else {
                combined_source.push_str(&imported_code);
            }
            combined_source.push('\n');

        } else {
            combined_source.push_str(line);
            combined_source.push('\n');
        }
    }

    combined_source.push_str(&format!("\n# --- END IMPORT: {} ---\n", file_name));
    Ok(combined_source)
}

fn parse_import_line(line: &str, line_num: usize) -> Result<(String, Option<String>), String> {
    let mut raw_args = if line.starts_with("import=") {
        line[7..].trim()
    } else if line.starts_with("import") {
        line[6..].trim()
    } else {
        line
    };

    if raw_args.starts_with('=') {
        raw_args = raw_args[1..].trim();
    }

    let mut value_part = raw_args;
    let mut alias_opt = None;

    if let Some(idx) = value_part.rfind(" as ") {
        let alias = value_part[idx+4..].trim();
        if alias.chars().all(|c| c.is_alphanumeric() || c == '_') {
             alias_opt = Some(alias.to_string());
             value_part = value_part[..idx].trim();
        }
    }

    if (value_part.starts_with('\'') && value_part.ends_with('\'')) ||
       (value_part.starts_with('"') && value_part.ends_with('"')) {
        Ok((value_part[1..value_part.len()-1].to_string(), alias_opt))
    } else {
        Err(format!("Line {}: Import path must be quoted.", line_num))
    }
}
