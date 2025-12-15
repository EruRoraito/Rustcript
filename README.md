<p align="center">
  <img src="logo.png" alt="rustcript logo" width="600"/>
</p>

<div align="center">
  <h1>Rustscript</h1>
  <p><strong>A companion scripting language for Rust projects</strong></p>

  ![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
</div>


## 1. About the Project

**Rustcript** is a robust, embeddable scripting language written in native Rust, developed primarily for deep integration within Rust projects. It is designed to add complex and changeable features to the applications without needing to recompile the core Rust binary.

Rustcript is complimentary to Rust projects, adding dynamic flexibility in an easy way without needing to change the core app code. The primary goal of the project is to provide a flexible scripting language for Rust projects that is easy enough for non-programmers to understand and use.

Without Rustcript, developers would need to hardcode dynamic business logic, rules, and complex configurations directly into the core Rust application. This approach means that every minor change, whether to a data processing rule, an event handling workflow, or a configuration parameter, requires a full re-compilation and re-deployment of the entire service. Rustcript solves this problem by separating the high-performance core logic (the "engine," written in Rust) from the dynamic, changeable business logic (the "external brain," written in Rustscript). This allows developers and even non-programmers to update and deploy new logic on-the-fly, dramatically increasing flexibility and reducing development cycles.

It is designed to be both secure and flexible, allowing developers to tailor its capabilities based on their needs. The basic version provides standard structured programming abilities, while an extended version can be compiled with access to I/O and OS-level APIs for external automation tasks.

#### For more technical information, please refer to the [MANUAL.md](MANUAL.md).
---

## 2. Full Features

*   **Flexible & Intuitive Syntax:** Combines natural assignment (`x = 10`) with a clear, space-separated command structure (`print 'hello'`, `if x > 10 [...]`).
*   **Rich Type System:** Supports integers, floats, booleans, strings, tuples, vectors, and hashmaps.
*   **Structured Control Flow:** Modern `if/else`, `match`, `while`, `for`, `foreach`, and `try/catch` blocks.
*   **Full-Fledged Functions:** Supports arguments, return values, scope isolation, and recursion.
*   **First-Class Functions:** Functions can be stored in variables and passed as arguments, enabling higher-order programming patterns.
*   **Seamless Rust Interoperability:** Safely wrap native Rust structs and expose their fields and methods directly to the script via the `RustcriptObject` trait.
*   **Modular by Design:** Import script files into others, with support for namespacing to prevent collisions.
*   **Comprehensive Standard Library:** Built-in methods for strings, collections, math, randomness, and JSON serialization.
*   **Security First:** Potentially dangerous features like OS command execution and file system access are **disabled by default** and must be explicitly enabled at compile time.
*   **Sandboxed & Permissive File I/O:** When enabled, all file operations are off by default. Permissions must be granted granularly (`read`, `write`, `delete`), and operations are confined to a designated directory, preventing unauthorized file access.
*   **Execution Safety:** An instruction counter prevents infinite loops from hanging the host application. This limit is fully configurable via CLI flags or the library API.
*   **Improved Error Reporting:** Runtime errors are reported with the **exact source line number**, greatly simplifying the debugging of complex scripts.

### Key Advantages for Embedders
*   **Safe Native Interop:** You can expose your Rust structs (application entities, database connections, config objects) directly to scripts using the `RustcriptObject` trait.
*   **Sandboxed & Secure:** By default, scripts cannot touch the filesystem or shell. You explicitly grant permissions (`file_io`, `os_access`) or define a sandbox root directory.
*   **Resilient:** The built-in instruction limiter prevents infinite loops from freezing your main application.
*   **No Garbage Collector:** It relies on Rust's memory management (`Arc`/`Clone`), making behavior predictable.
---

## 2. Use Cases


**Rustcript** excels as a "glue language" used to orchestrate powerful, high-performance Rust components.

| Use Case                  | Description                                                                                                                   |
| :------------------------ | :---------------------------------------------------------------------------------------------------------------------------- |
| **Embedded Scripting**    | Define application behavior (quests, dialogues, AI) that can be changed without recompiling the main program.                 |
| **ETL & Data Pipelines**  | Automate complex data processing workflows: extract, transform, and load data using the sandboxed `io` module.                |
| **Automated Testing**     | Write readable, step-by-step test cases for applications, APIs, or CLI tools using the gated `os.exec` module.                |
| **Dynamic Configuration** | Go beyond static `.ini` or `.json` files by using scripts to define configuration that changes based on environment or logic. |
| **Report Generation**     | Define the structure and content of automated reports, pulling data from various sources and formatting it dynamically.         |
| **Microservice Logic** | Implement business logic (e.g., request validation, data transformation, routing) within a high-performance Rust microservice. The core service handles infrastructure, while scripts define the behavior, allowing for on-the-fly logic updates without recompiling or restarting. |
| **Event-Driven Processing**  | Define the logic for event consumers in `rustcript`. Scripts can dynamically filter, enrich, route, and process messages from queues (e.g., Kafka, RabbitMQ), enabling rapid iteration on event handling rules without redeploying the core consumer application. |

---

## 3. Getting Started

### Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install) toolchain installed.

### Build & Run

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Eng-AliKazemi/Rustcript.git
    cd rustcript
    ```

2.  **Build the interpreter:**
    For basic functionality:
    ```bash
    cargo build --release
    ```
    To include all features (recommended for running the full test suite):
    ```bash
    cargo build --release --features "os_access file_io"
    ```

3.  **Run an example script:**
    ```bash
    ./target/release/rustcript examples/01_basics.rc
    ```

### Command-Line Arguments

| Flag                  | Argument | Description                                                                                             |
| :-------------------- | :------- | :------------------------------------------------------------------------------------------------------ |
| `--limit`             | `<N>`    | Sets a maximum instruction limit to `N`. The script will abort if it exceeds this, preventing infinite loops. |
| `--unlimited`         | (None)   | Disables the execution safety limit entirely. Use with caution.                                              |
| `--sandbox`           | `<PATH>` | **(Requires `file_io` feature)** Enables the `io` module and restricts all file operations to the specified directory path. |
| `--allow-read`        | (None)   | **(Requires `file_io` feature)** Grants permission for `io.read` and `io.exists`.                       |
| `--allow-write`       | (None)   | **(Requires `file_io` feature)** Grants permission for `io.write` and `io.append`.                      |
| `--allow-delete`      | (None)   | **(Requires `file_io` feature)** Grants permission for `io.delete`.                                     |
| `--unsafe-no-sandbox` | (None)   | **(Requires `file_io` feature)** Disables sandbox protection, allowing I/O on the entire host filesystem. **Use with extreme caution.** |
| `--help`              | (None)   | Displays the help message with all available options.                                                   |


**Example:**
```bash
# Run a script with a strict limit and a file sandbox with read/write permissions
./target/release/rustcript --limit 10000 --sandbox ./data --allow-read --allow-write data_processing.rc
```
---

## 4. Test Suits
The project includes a comprehensive test suite to ensure correctness and prevent regressions. It is divided into two parts: Rust-native tests for the core library and a script-based suite for end-to-end validation.

### Part 1: Rust Unit & Integration Tests

These tests validate the core library functionality, such as the parser, type system, and the public API for native interoperability. They are essential for verifying the internal logic.

*   **Run all library tests:**
    ```bash
    cargo test
    ```
*   **Run tests with detailed output (for debugging):**
    ```bash
    cargo test -- --nocapture
    ```

### Part 2: End-to-End Scripting Tests

The `tests/test_runner.rc` script executes all example scripts to verify the interpreter's behavior from a user's perspective. This is a crucial regression test for the language itself.

**Note:** This test suite requires both the `os_access` and `file_io` features because the runner script calls the compiled binary as a subprocess and tests the file I/O example.

1.  **Build the interpreter with the required features:**
    This compilation step is required before running the script-based tests.
    ```bash
    cargo build --release --features "os_access file_io"
    ```

2.  **Execute the test suite with different configurations:**

    *   **Standard Run (Default Safety Limit):**
        This command runs the test suite with the default instruction limit (`1,000,000`). It ensures all standard scripts execute correctly and that the infinite loop test (`22_safety_limit.rc`) is properly terminated.
        ```bash
        ./target/release/rustcript tests/test_runner.rc
        ```

    *   **Unlimited Run (No Safety Limit):**
        This verifies that the interpreter functions correctly when the safety counter is disabled. The infinite loop test is expected to fail (and be caught by the test runner) in this mode.
        ```bash
        ./target/release/rustcript --unlimited tests/test_runner.rc
        ```



---
## 5. Architecture

The codebase is organized as a library crate with a thin binary client, following SOLID principles for high maintainability. Each module has a single, well-defined responsibility.

| File                   | Description                                                                                                   |
| :--------------------- | :------------------------------------------------------------------------------------------------------------ |
| **Core Engine**        |                                                                                                               |
| `lib.rs`               | The library crate root. Exports the public API (`Interpreter`, `Value`, etc.) for embedding.                  |
| `main.rs`              | The binary crate entry point. A thin CLI client that uses the `Rustcript` library to execute scripts from files. |
| `interpreter.rs`       | The main execution engine. Manages runtime state, including scopes, stacks, and safety limits.                |
| `interpreter_step.rs`  | The single-step execution dispatcher. Takes one `Statement` and delegates it to the appropriate handler.        |
| `interpreter_utils.rs` | Provides helpers for parsing and mutating complex access chains (e.g., `obj.prop[0]`).                        |
| **Language Definition**|                                                                                                               |
| `types.rs`             | Defines core data structures like `Program`, the `Statement` enum, and the `ScriptHandler` trait.             |
| `parser.rs`            | Transforms source code into a vector of `Statement` enums and a jump map for control flow.                    |
| `data_types.rs`        | Defines the dynamic `Value` enum, representing all script-level data types (int, string, vec, etc.).          |
| `complex_types.rs`     | A helper module that handles parsing of nested structures like `(...)`, `[...]`, and `{...}`.                 |
| `operators.rs`         | Implements logic for all arithmetic (`+`), comparison (`==`), and logical (`&&`) operators.                   |
| **Control Flow**       |                                                                                                               |
| `flow_control.rs`      | Handles branching logic for `if`, `else`, `goto`, and `match` statements.                                     |
| `loops.rs`             | Handles the execution logic for `while`, `for`, and `foreach` loops.                                          |
| `match_control.rs`     | A specialized handler for the `match/case/default` control flow structure.                                    |
| **Features & Libraries** |                                                                                                               |
| `importer.rs`          | Resolves `import '...'` statements recursively, handling file reading, cycle detection, and namespacing.     |
| `functions.rs`         | Contains helpers for parsing function definitions/calls and binding arguments to parameters.                  |
| `user_data.rs`         | Defines the `RustcriptObject` trait, the public interface for exposing native Rust structs to the script.      |
| `stdlib.rs`            | The standard library. Dispatches method calls (`vec.push`) and static module calls (`math.sqrt`).             |
| `io_lib.rs`            | **(Feature: `file_io`)** Implements the sandboxed file system API (`io.read`, `io.write`, etc.).               |
| `json_lib.rs`          | Implements `json.parse` and `json.stringify` functionality using `serde_json`.                                |
| `regex_lib.rs`         | Implements regex methods for strings (`is_match`, `find_all`) using the `regex` crate.                          |

---

## 6. Compilation & Integration

`Rustcript` is fundamentally a library crate, designed to be embedded as a scripting engine within a larger host application. While a standalone interpreter binary is provided for testing and direct script execution, the primary and most powerful use case is integrating the `rustcript` crate into the projects.

There are two primary methods for integrating the `Rustcript` library:

### Method 1: Direct Integration via Cargo (Recommended for Rust Projects)

The easiest and most robust way to use `Rustcript` in another Rust project is to add it as a local `path` dependency in your project's `Cargo.toml`. This allows Cargo to manage the compilation process, ensuring that compiler versions, flags, and dependencies are perfectly aligned.

**In your host project's `Cargo.toml`:**
```toml
[package]
name = "my_awesome_app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Point this to your local rustcript folder.
# Cargo will compile it as a library and link it automatically.
rustcript = { path = "/path/to/rustcript" }
```

Once added, you can use `rustcript`'s public API directly in your code:
```rust
use rustcript::{Interpreter, Value, RustcriptObject};
// ... your application logic ...
```

### Method 2: Foreign Function Interface (FFI) for Non-Rust Projects

For integration with other languages (like Python, C#, C++, Go, etc.), you can compile `rustcript` into a C-compatible dynamic library (`.so` on Linux, `.dll` on Windows, `.dylib` on macOS).

This involves two steps:

1.  **Expose a C-compatible API:** You would create a wrapper with `#[no_mangle]` and `extern "C"` functions to safely expose the interpreter's functionality. For example:

    ```rust
    // In rustcript's lib.rs or a new ffi.rs module
    use rustcript::Interpreter;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    #[no_mangle]
    pub extern "C" fn rustcript_run_script(source_code: *const c_char) -> *mut c_char {
        // Unsafe block to handle raw pointers from C
        let c_str = unsafe { CStr::from_ptr(source_code) };
        let source = c_str.to_str().unwrap_or("");

        // ... (Create interpreter, handler, and run) ...

        let result_string = "OK".to_string(); // Or an error message
        CString::new(result_string).unwrap().into_raw()
    }
    ```

2.  **Compile as a `cdylib`:** Modify `rustcript`'s `Cargo.toml` to produce a dynamic library:
    ```toml
    [lib]
    crate-type = ["cdylib"]
    ```

This is an advanced use case that requires careful handling of memory and data types across the language boundary.

## 7. Binary Size Optimization

To achieve the smallest possible release binary size for your host executable or the FFI library, apply the following optimization profile to your `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"     # Optimize for size ('z' is more aggressive than 's')
lto = true          # Enable Link Time Optimization (removes dead code across crates)
codegen-units = 1   # Compile as a single unit (maximizes optimization, slower build time)
panic = "abort"     # Removes stack unwinding info (saves space, but panics crash immediately)
strip = true        # Automatically strip symbols from the binary (requires Rust 1.59+)
```

These aggressive settings prioritize size over compilation speed and runtime panic verbosity, resulting in a significantly leaner compiled artifact.

## 8. AI-Powered Development

To accelerate development and enable automated script generation, this project includes a comprehensive knowledge base file ("Prompt Engineering") specifically designed to be fed to Large Language Models (LLMs).

#### [Click here to view the AI Instruction Manual](AI_INSTRUCTIONS.md)

The, `AI_INSTRUCTIONS.md`, is a single-source-of-truth document that teaches an AI to become an expert `Rustcript` developer.

### Features for AI Integration
*   **System Prompting & Persona:** The file begins with a system prompt that primes the AI to act as a specialist in the `Rustcript` language and its interpreter.
*   **Deep Architectural Insights:** It goes beyond the public manual, explaining the *internal logic* of the interpreter—how the parser builds an AST, how the `jump_map` optimizes control flow, and how the state machine manages scopes and stacks. This context allows the AI to reason about the code from first principles.
*   **Formal Grammar & API Reference:** The complete language syntax and standard library are documented in a structured, machine-readable format, minimizing ambiguity.
*   **Few-Shot Learning Zone:** The document contains a rich set of examples formatted as `Task -> Thought Process -> Correct Code`. This teaches the AI not just *what* the correct code is, but *how* to reason about a problem and arrive at the solution, a technique derived from Chain-of-Thought prompting.
*   **Anti-Pattern Mitigations:** A dedicated section lists common mistakes and invalid syntax patterns, training the AI to avoid them proactively.

### Use Case
By providing `AI_INSTRUCTIONS.md` as context to a capable LLM, developers can:
*   **Generate Scripts from Natural Language:** "Write a script that reads a JSON file, filters for users with more than 100 points, and writes their names to an output file."
*   **Debug and Refactor Code:** "This script is throwing a 'Variable not found' error on line 15. Can you fix it?"
*   **Translate Logic:** "Convert this Python dictionary manipulation logic into the equivalent Rustcript code."
*   **Automate Documentation:** "Generate a summary of what this complex `.rc` script does."

This approach turns a modern AI into a powerful pair-programmer, fully versed in the specifics and best practices of the Rustcript language.

---

## 9. Project Philosophy & Future Roadmap

### From a Personal Tool to an Open-Source Project

This project represents the first public version of Rustcript. It began as a practical solution to add dynamic flexibility to my own Rust applications, allowing me to modify logic and configurations without the friction of a full re-compilation for every minor change.

During its development, Rustcript evolved significantly. What started as a simple need for dynamic scripting grew into a robust tool with a comprehensive security model, a full-fledged standard library, and a powerful Rust interoperability layer, doing much more than its initial scope required. The result is a powerful, secure, and resilient interpreter that is already well-suited for its primary use cases.

### The Future is Community-Driven

While the current version is stable and feature-rich, the goal is to see Rustcript grow to serve a wider range of applications and needs in the future. The roadmap will be heavily influenced by the community.

We strongly encourage users and contributors to help shape the future of Rustcript. If you have an idea for a new feature, a suggestion for improving the language syntax, or have found a bug, please open a GitHub Issue. Your feedback is crucial for this next phase of development.

**Have a feature request? You can open a new issue here:**
[**https://github.com/Eng-AliKazemi/Rustcript/issues/new**](https://github.com/Eng-AliKazemi/Rustcript/issues/new)

---

## 10. How to Contribute

We welcome contributions from the community! Whether it's reporting a bug, improving documentation, suggesting a new feature, or writing code, your help is greatly appreciated in making Rustcript better for everyone.

### Types of Contributions We're Looking For

*   **Code Contributions:** Fixing bugs or implementing new language features, standard library functions, or interpreter optimizations.
*   **Documentation:** Improving the [`README.md`](README.md), the [`MANUAL.md`](MANUAL.md), the [`AI_INSTRUCTIONS.md`](AI_INSTRUCTIONS.md), or inline code comments.*   **Bug Reports & Feature Requests:** Submitting detailed issues and well-thought-out ideas in the [Issues tab](https://github.com/Eng-AliKazemi/Rustcript/issues/new).

### General Contribution Workflow

To ensure a smooth and collaborative process for code changes, we follow a simple guideline:

**➡️ Please open a GitHub Issue to discuss your idea *before* starting to write code.**

This approach helps us:
-   **Align on Goals:** Ensure your proposed change fits with the project's vision and roadmap.
-   **Avoid Duplicate Work:** Check if someone else is already working on a similar feature.
-   **Refine the Technical Approach:** Discuss the best way to implement your idea and get early feedback.
-   **Streamline the Review Process:** Make the pull request review much faster and more straightforward.

**Workflow Steps:**

1.  **Start a Discussion:** Go to the [**Discussions tab**](https://github.com/Eng-AliKazemi/Rustcript/discussions) and open a new topic. Clearly describe the bug you want to fix or the feature you want to add. For new features, please outline the proposal and its use case. We'll use the discussion to refine the scope and plan before an issue is created.

2.  **Fork & Branch:** Once the idea is discussed and agreed upon, fork the repository and create a new branch for your work. A good branch name is descriptive. **Please always work on a feature branch, not on your fork's `master` branch.** This keeps your main branch clean and makes the PR process smoother.
    ```bash
    # Create a new branch from the 'development' branch
    git checkout development
    git pull origin development
    git checkout -b feature/your-amazing-feature
    ```

3.  **Develop & Test:** Make your changes, adhering to the project's coding style and Rust best practices. **Crucially, ensure you test your changes thoroughly.**

4.  **Submit a Pull Request:** Push your feature branch to your fork and open a pull request against the **`development`** branch of the `Eng-AliKazemi/Rustcript` repository. Pull requests should always come from your feature branch, not your `master` branch. Please provide a clear description of your changes in the PR and link it to the original discussion or issue.

We look forward to collaborating with you

---

## 11. License

This project is licensed under the **Mozilla Public License 2.0 (MPL-2.0)**.

You are free to use, study, share, and modify the software. If you modify any MPL-licensed files, you must make the source code of those specific files available. You may combine this software with proprietary code in a larger project without needing to release the source code of your other components.

### Additional Attribution Requirement
In accordance with MPL 2.0, all distributions of this software in **binary and source form** must include the following user-visible attribution in at least one of the following locations:
- In the README
- On a startup/splash screen
- In an "About" or "Acknowledgements" dialog box
- In the primary documentation or "Help" menu
- As a footer on a command-line tool's initial output

**Required attribution text:**
> Powered by Rustcript from https://github.com/Eng-AliKazemi/Rustcript.

For the full license text, please see the [`LICENSE`](LICENSE) file.

---

## 12. Code of Conduct

This project and everyone participating in it is governed by the [Ruscript Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

---

## 13. Acknowledgments

The development of Rustcript relies on the exceptional work of the open-source community. We extend our deepest gratitude to the developers behind the incredible Rust crates that made this interpreter possible, including:

For a complete list of third-party dependencies and their versions, please see the `Cargo.toml` file.

---

## 14. Contact

To get in touch with the developer, please use the following link:

<a href="https://linkedin.com/in/e-a-k" target="_blank"><img src="https://img.shields.io/badge/Connect-LinkedIn-0077B5?style=for-the-badge&logo=linkedin&logoColor=white&labelColor=555" alt="Connect on LinkedIn"/></a>
