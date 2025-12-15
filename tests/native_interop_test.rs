// File Version: 1.0.0
// /tests/native_interop_test.rs

use rustcript::{Interpreter, ScriptHandler, Value, RustcriptObject};
use std::sync::{Arc, Mutex};

// --- 1. Define a Native Rust Struct ---
#[derive(Debug)]
struct GameCharacter {
    name: String,
    hp: i32,
    max_hp: i32,
}

// --- 2. Implement the Interop Trait ---
impl RustcriptObject for GameCharacter {
    fn type_name(&self) -> &str {
        "GameCharacter"
    }

    // Handle "obj.prop"
    fn get(&self, field: &str) -> Option<Value> {
        match field {
            "name" => Some(Value::String(self.name.clone())),
            "hp" => Some(Value::Integer(self.hp)),
            "max_hp" => Some(Value::Integer(self.max_hp)),
            _ => None
        }
    }

    // Handle "obj.prop = val"
    fn set(&mut self, field: &str, value: Value) -> Result<(), String> {
        match field {
            "name" => {
                self.name = value.to_string();
                Ok(())
            },
            "hp" => {
                if let Ok(v) = value.as_float() {
                    self.hp = v as i32;
                    Ok(())
                } else {
                    Err("HP must be a number".to_string())
                }
            },
            _ => Err(format!("Field '{}' is read-only or does not exist", field))
        }
    }

    // Handle "obj.method(args)"
    fn call(&mut self, method: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
        match method {
            "heal" => {
                if args.len() != 1 {
                    return Err("heal expects 1 argument (amount)".to_string());
                }
                let amount = args[0].as_float().unwrap_or(0.0) as i32;
                self.hp = (self.hp + amount).min(self.max_hp);
                Ok(Some(Value::Integer(self.hp)))
            },
            "take_damage" => {
                if args.len() != 1 {
                    return Err("take_damage expects 1 argument (amount)".to_string());
                }
                let amount = args[0].as_float().unwrap_or(0.0) as i32;
                self.hp = (self.hp - amount).max(0);
                Ok(Some(Value::Integer(self.hp)))
            },
            "is_alive" => {
                Ok(Some(Value::Boolean(self.hp > 0)))
            },
            _ => Err(format!("Method '{}' not implemented", method))
        }
    }
}

// --- 3. Mock Handler for Output Capture ---
struct TestHandler {
    output: Vec<String>,
}
impl TestHandler {
    fn new() -> Self { Self { output: Vec::new() } }
}
impl ScriptHandler for TestHandler {
    fn on_print(&mut self, text: &str) { self.output.push(text.to_string()); }
    fn on_input(&mut self, _v: &str) -> String { "".to_string() }
    fn on_command(&mut self, _c: &str, _a: Vec<&str>) -> Result<bool, String> { Ok(true) }
}

#[test]
fn test_rust_interop() {
    // A. Setup Native Object
    let hero = GameCharacter { name: "rustcript Warrior".to_string(), hp: 50, max_hp: 100 };
    // Wrap in Arc<Mutex> and Value
    let hero_val = Value::UserData(Arc::new(Mutex::new(hero)));

    // B. Inject into Interpreter
    let src = "
        print='Starting: {hero.name} (HP: {hero.hp})'

        # 1. Modify Property
        hero.name = 'Super Warrior'
        print='Renamed: {hero.name}'

        # 2. Call Method (Heal)
        method=new_hp = hero.heal(20)
        print='Healed to: {new_hp}'
        print='Verify Property: {hero.hp}'

        # 3. Call Method (Damage)
        method=hero.take_damage(60)
        print='Taken Damage: {hero.hp}'

        # 4. Check Boolean Logic with Method
        method=alive = hero.is_alive()
        print='Is Alive? {alive}'
    ";

    let mut interp = Interpreter::from_source(src).unwrap();
    interp.set_global("hero", hero_val);

    let mut handler = TestHandler::new();
    interp.run(&mut handler).expect("Script execution failed");

    // C. Verify Output
    assert_eq!(handler.output[0], "Starting: rustcript Warrior (HP: 50)");
    assert_eq!(handler.output[1], "Renamed: Super Warrior");
    assert_eq!(handler.output[2], "Healed to: 70");
    assert_eq!(handler.output[3], "Verify Property: 70");
    assert_eq!(handler.output[4], "Taken Damage: 10");
    assert_eq!(handler.output[5], "Is Alive? true");
}
