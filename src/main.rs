// File Version: 4.4.0
// /src/main.rs

use rustcript::{Interpreter, ScriptHandler, resolve_imports};
use rustcript::types::IoPermissions;

use std::env;
use std::io::{self, Write};
use std::process;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;

const DEFAULT_OP_LIMIT: usize = 1_000_000;

struct ConsoleHandler;
impl ScriptHandler for ConsoleHandler {
    fn on_print(&mut self, text: &str) {
        println!("{}", text);
        let _ = io::stdout().flush();
    }

    fn on_input(&mut self, _var: &str) -> String {
        print!("> ");
        let _ = io::stdout().flush();
        let mut buf = String::new();
        match io::stdin().read_line(&mut buf) {
            Ok(_) => buf.trim().to_string(),
            Err(_) => String::new(),
        }
    }

    fn on_command(&mut self, cmd: &str, args: Vec<&str>) -> Result<bool, String> {
        match cmd {
            "wait" => {
                let ms = args.get(0).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
                thread::sleep(Duration::from_millis(ms));
                Ok(true)
            }
            "beep" => {
                println!("[BEEP]");
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}

struct Config {
    script_file: String,
    limit: usize,
    sandbox_path: Option<String>,
    io_perms: IoPermissions,
}

impl Config {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut script_file: Option<String> = None;
        let mut explicit_limit: Option<usize> = None;
        let mut sandbox_path: Option<String> = None;
        let mut io_perms = IoPermissions::default();

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];
            match arg.as_str() {
                "--help" | "-h" => {
                    print_usage(&args[0]);
                    process::exit(0);
                }
                "--unlimited" => {
                    explicit_limit = Some(0);
                }
                "--limit" => {
                    i += 1;
                    if i >= args.len() { return Err("--limit requires a number".to_string()); }
                    explicit_limit = Some(args[i].parse().map_err(|_| "Invalid number for --limit")?);
                }
                "--sandbox" => {
                    i += 1;
                    if i >= args.len() { return Err("--sandbox requires a path".to_string()); }
                    sandbox_path = Some(args[i].clone());
                }
                "--allow-read" => io_perms.read = true,
                "--allow-write" => io_perms.write = true,
                "--allow-delete" => io_perms.delete = true,
                "--unsafe-no-sandbox" => io_perms.allow_no_sandbox = true,
                _ => {
                    if arg.starts_with('-') {
                        return Err(format!("Unknown option: {}", arg));
                    } else {
                        script_file = Some(arg.clone());
                    }
                }
            }
            i += 1;
        }

        let script_file = script_file.ok_or("No input file specified.")?;

        let limit = explicit_limit.unwrap_or_else(|| {
            env::var("rustcript_MAX_OPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_OP_LIMIT)
        });

        Ok(Config {
            script_file,
            limit,
            sandbox_path,
            io_perms,
        })
    }
}

fn print_usage(program_name: &str) {
    eprintln!("rustcript Interpreter v0.1.0");
    eprintln!("Usage: {} [options] <file.rc>", program_name);
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --limit <N>      Set max instruction count (overrides env var)");
    eprintln!("  --unlimited      Disable execution safety limit");
    eprintln!("  --sandbox <PATH> Set the root directory for File I/O (Requires feature 'file_io')");
    eprintln!("  --help           Show this message");
    eprintln!("");
    eprintln!("I/O Permissions (Requires feature 'file_io'):");
    eprintln!("  --allow-read     Enable file reading");
    eprintln!("  --allow-write    Enable file writing");
    eprintln!("  --allow-delete   Enable file deletion");
    eprintln!("  --unsafe-no-sandbox  DISABLE SANDBOX (Allow access to host filesystem)");
    eprintln!("");
    eprintln!("Environment Variables:");
    eprintln!("  rustcript_MAX_OPS Set default max instruction count (Default: 1,000,000)");
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        return Err("No arguments provided".to_string());
    }

    let config = Config::parse(args)?;

    let src = resolve_imports(&config.script_file)
        .map_err(|e| format!("Import Error: {}", e))?;

    let mut interp = Interpreter::from_source(&src)
        .map_err(|e| format!("Parse Error: {}", e))?;

    interp.set_instruction_limit(config.limit);
    interp.set_io_permissions(config.io_perms);

    if let Some(path) = config.sandbox_path {
        interp.set_sandbox_root(PathBuf::from(path));
    }

    interp.run(&mut ConsoleHandler).map_err(|e| format!("Runtime Error: {}", e))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
