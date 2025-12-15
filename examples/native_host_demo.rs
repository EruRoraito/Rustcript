// File Version: 1.2.0
// /examples/native_host_demo.rs

use rustcript::{Interpreter, RustcriptObject, ScriptHandler, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// --- 1. Define Your Rust Struct ---
#[derive(Debug)]
struct MockDatabase {
    store: HashMap<String, String>,
    connected: bool,
}

// --- 2. Implement RustcriptObject ---
impl RustcriptObject for MockDatabase {
    fn type_name(&self) -> &str {
        "DatabaseConnection"
    }

    fn get(&self, field: &str) -> Option<Value> {
        match field {
            "connected" => Some(Value::Boolean(self.connected)),
            "count" => Some(Value::Integer(self.store.len() as i32)),
            _ => None,
        }
    }

    fn call(&mut self, method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
        match method {
            "connect" => {
                self.connected = true;
                println!("[Host] Database Connected!");
                Ok(None)
            },
            "disconnect" => {
                self.connected = false;
                println!("[Host] Database Disconnected!");
                Ok(None)
            },
            "insert" => {
                if !self.connected { return Err("Database not connected".to_string()); }
                if args.len() != 2 { return Err("insert(key, val) requires 2 args".to_string()); }

                let key = args[0].to_string();
                let val = args[1].to_string();
                self.store.insert(key, val);
                Ok(None)
            },
            "query" => {
                if !self.connected { return Err("Database not connected".to_string()); }
                if args.len() != 1 { return Err("query(key) requires 1 arg".to_string()); }

                let key = args[0].to_string();
                match self.store.get(&key) {
                    Some(v) => Ok(Some(Value::String(v.clone()))),
                    None => Ok(Some(Value::String("NULL".to_string()))),
                }
            },
            _ => Err(format!("Unknown method: {}", method)),
        }
    }
}

// --- 3. Output Handler ---
struct DemoHandler;
impl ScriptHandler for DemoHandler {
    fn on_print(&mut self, text: &str) { println!("[Script] {}", text); }
    fn on_input(&mut self, _: &str) -> String { String::new() }
    fn on_command(&mut self, _: &str, _: Vec<&str>) -> Result<bool, String> { Ok(true) }
}

fn main() {
    println!("--- rustcript Native Interop Demo (Natural Syntax) ---");

    let db = MockDatabase { store: HashMap::new(), connected: false };
    let db_val = Value::UserData(Arc::new(Mutex::new(db)));

    // NOTE: We no longer use 'method=' prefix!
    let script = r#"
        print='1. Check Status...'
        print='   Is Connected? {db.connected}'

        print='2. Connecting (Natural Call)...'
        db.connect()

        print='3. Inserting Data...'
        db.insert('user_1', 'Alice')
        db.insert('user_2', 'Bob')

        print='   Rows: {db.count}'

        print='4. Querying Data...'
        u1 = db.query('user_1')
        print='   User 1 is: {u1}'

        print='5. Disconnecting...'
        db.disconnect()
    "#;

    let mut interpreter = Interpreter::from_source(script).unwrap();
    interpreter.set_global("db", db_val);

    let mut handler = DemoHandler;
    if let Err(e) = interpreter.run(&mut handler) {
        eprintln!("Error: {}", e);
    }
}
