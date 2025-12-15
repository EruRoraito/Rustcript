// File Version: 1.2.0
// /tests/lib_integration_test.rs

use rustcript::{Interpreter, ScriptHandler, Value};

// 1. Define a Dummy Handler for testing
// This captures output instead of printing to console
struct TestHandler {
    pub output: Vec<String>,
}

impl TestHandler {
    fn new() -> Self {
        Self { output: Vec::new() }
    }
}

impl ScriptHandler for TestHandler {
    fn on_print(&mut self, text: &str) {
        self.output.push(text.to_string());
    }

    fn on_input(&mut self, _var: &str) -> String {
        "test_input".to_string()
    }

    fn on_command(&mut self, _cmd: &str, _args: Vec<&str>) -> Result<bool, String> {
        Ok(true)
    }
}

#[test]
fn test_infinite_loop_safety() {
    // A script that loops forever
    // Updated to new syntax: 'while true' instead of 'while=true'
    let src = "
        counter = 0
        while true [
            counter += 1
        ]
    ";

    let mut interp = Interpreter::from_source(src).unwrap();
    let mut handler = TestHandler::new();

    // Set a strict limit of 100 instructions
    interp.set_instruction_limit(100);

    // Run
    let result = interp.run(&mut handler);

    // Assert that it failed specifically due to the limit
    assert!(result.is_err());
    let err_msg = result.err().unwrap();
    assert!(err_msg.contains("Execution Limit Exceeded"));
}

#[test]
fn test_state_injection_and_extraction() {
    // 1. Create Interpreter
    // Updated to new syntax: 'print ...'
    let src = "
        # Read the injected global
        print 'Hello, {USER}!'

        # Modify it
        # Syntax: target operand1 op operand2
        result_val USER_ID * 2
    ";
    let mut interp = Interpreter::from_source(src).unwrap();
    let mut handler = TestHandler::new();

    // 2. Inject Data (Simulating a host application configuration)
    interp.set_global("USER", Value::String("Tester".to_string()));
    interp.set_global("USER_ID", Value::Integer(21));

    // 3. Run
    interp.run(&mut handler).expect("Script failed");

    // 4. Verify Output
    assert_eq!(handler.output[0], "Hello, Tester!");

    // 5. Extract Result (Simulating reading back data)
    let result = interp.get_value("result_val").expect("Variable not found");

    if let Value::Integer(i) = result {
        assert_eq!(i, 42); // 21 * 2
    } else {
        panic!("Expected Integer, got {:?}", result);
    }
}

#[test]
fn test_default_is_unlimited() {
    // Updated to new syntax
    let src = "
        i = 0
        while i < 2000 [
            i += 1
        ]
    ";

    let mut interp = Interpreter::from_source(src).unwrap();
    // Do NOT set limit

    let mut handler = TestHandler::new();
    let result = interp.run(&mut handler);

    assert!(result.is_ok());
}
