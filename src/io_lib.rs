// File Version: 2.1.0
// /src/io_lib.rs

use crate::data_types::Value;
use crate::types::IoPermissions;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

fn resolve_safe_path(root: Option<&Path>, perms: &IoPermissions, user_path: &str) -> Result<PathBuf, String> {
    let path = Path::new(user_path);

    if perms.allow_no_sandbox {
        return Ok(path.to_path_buf());
    }

    let sandbox_root = root.ok_or_else(|| "File I/O Error: Sandbox path not configured.".to_string())?;

    if path.is_absolute() {
        return Err("Security Violation: Absolute paths are not allowed in sandbox mode.".to_string());
    }

    let candidate = sandbox_root.join(path);

    let parent = candidate.parent()
        .ok_or_else(|| "Invalid file path: No parent directory".to_string())?;

    let canon_parent = parent.canonicalize()
        .map_err(|e| format!("Directory does not exist or access denied: {}", e))?;

    let canon_root = sandbox_root.canonicalize()
        .map_err(|e| format!("Sandbox root error: {}", e))?;

    if !canon_parent.starts_with(&canon_root) {
        return Err("Security Violation: Path traversal detected.".to_string());
    }

    Ok(candidate)
}

fn require_perm(allowed: bool, action: &str) -> Result<(), String> {
    if allowed {
        Ok(())
    } else {
        Err(format!("Security Violation: {} permission denied.", action))
    }
}

fn get_write_args(args: &[Value], method_name: &str) -> Result<(String, String), String> {
    if args.len() != 2 {
        return Err(format!("io.{} expects 2 arguments (filename, content)", method_name));
    }
    Ok((args[0].to_string(), args[1].to_string()))
}

fn get_filename_arg(args: &[Value], method_name: &str) -> Result<String, String> {
    if args.len() != 1 {
        return Err(format!("io.{} expects 1 argument (filename)", method_name));
    }
    Ok(args[0].to_string())
}

pub fn handle_io(root: Option<&Path>, perms: &IoPermissions, method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
    match method {
        "write" => {
            require_perm(perms.write, "Write")?;
            let (filename, content) = get_write_args(&args, "write")?;
            let target_path = resolve_safe_path(root, perms, &filename)?;

            let mut file = fs::File::create(&target_path)
                .map_err(|e| format!("Failed to create file: {}", e))?;

            file.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to file: {}", e))?;

            Ok(Some(Value::Boolean(true)))
        },
        "append" => {
            require_perm(perms.write, "Write (Append)")?;
            let (filename, content) = get_write_args(&args, "append")?;
            let target_path = resolve_safe_path(root, perms, &filename)?;

            let mut file = fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&target_path)
                .map_err(|e| format!("Failed to open file for appending: {}", e))?;

            file.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to append to file: {}", e))?;

            Ok(Some(Value::Boolean(true)))
        },
        "read" => {
            require_perm(perms.read, "Read")?;
            let filename = get_filename_arg(&args, "read")?;
            let target_path = resolve_safe_path(root, perms, &filename)?;

            let canon_path = target_path.canonicalize()
                .map_err(|_| "File not found".to_string())?;

            if !perms.allow_no_sandbox {
                if let Some(s_root) = root {
                    let canon_root = s_root.canonicalize().unwrap_or_else(|_| PathBuf::from("."));
                    if !canon_path.starts_with(&canon_root) {
                        return Err("Security Violation: Path traversal detected via symlink.".to_string());
                    }
                }
            }

            let content = fs::read_to_string(canon_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            Ok(Some(Value::String(content)))
        },
        "exists" => {
            require_perm(perms.read, "Read (Exists)")?;
            let filename = get_filename_arg(&args, "exists")?;

            if let Ok(target_path) = resolve_safe_path(root, perms, &filename) {
                if target_path.exists() {
                    if !perms.allow_no_sandbox {
                        if let Ok(canon_path) = target_path.canonicalize() {
                            if let Some(s_root) = root {
                                if let Ok(canon_root) = s_root.canonicalize() {
                                    if canon_path.starts_with(canon_root) {
                                        return Ok(Some(Value::Boolean(true)));
                                    }
                                }
                            }
                        }
                        return Ok(Some(Value::Boolean(false)));
                    }
                    return Ok(Some(Value::Boolean(true)));
                }
            }
            Ok(Some(Value::Boolean(false)))
        },
        "delete" => {
            require_perm(perms.delete, "Delete")?;
            let filename = get_filename_arg(&args, "delete")?;
            let target_path = resolve_safe_path(root, perms, &filename)?;

            let canon_path = target_path.canonicalize()
               .map_err(|_| "File not found".to_string())?;

            if !perms.allow_no_sandbox {
                if let Some(s_root) = root {
                    let canon_root = s_root.canonicalize().unwrap();
                     if !canon_path.starts_with(&canon_root) {
                        return Err("Security Violation: Path traversal detected.".to_string());
                    }
                }
            }

            fs::remove_file(canon_path)
                .map_err(|e| format!("Failed to delete file: {}", e))?;

            Ok(Some(Value::Boolean(true)))
        },
        _ => Err(format!("Unknown method '{}' for io module", method)),
    }
}
