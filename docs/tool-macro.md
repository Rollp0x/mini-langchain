# Proc-macro: `#[tool(...)]`

This document specifies the design and minimal implementation plan for the `#[tool(...)]` attribute proc-macro used in `mini-langchain`.

Goals
- Allow users to annotate a plain Rust function and automatically generate a `Tool` wrapper (Params struct, JSON Schema helper and `Tool` impl).
- Keep v1 minimal and robust: support owned parameter types that implement `serde::Deserialize` and simple return types (e.g., `String`).
- Provide clear compile-time diagnostics for unsupported cases.

Attribute syntax (v1)

```rust
#[tool(
    name = "get_weather",                // optional; defaults to function name if omitted
    description = "Get weather for a city",
    params(
        city = "City name, e.g. 'San Francisco'",
        units = "celsius|fahrenheit"
    )
)]
fn get_weather(city: String, units: Option<String>) -> String { ... }
```

Notes
- `description` should be provided (macro emits an error or warning if missing).
- `params(...)` maps parameter names to descriptions. The macro uses function signature for types; every parameter declared in `params(...)` must exist in the function signature.
- Supported parameter types: owned types that implement `serde::DeserializeOwned` (String, numeric primitives, Option<T>, and user structs which derive Deserialize).
- Defaults: not parsed from function signature. If needed, users may provide defaults via `params(...)` metadata in a later version.

Generated artifacts (sketch)
- `#[derive(serde::Deserialize)] pub struct FnNameParams { ... }`
- `pub fn fn_name_schema() -> serde_json::Value { ... }` (optional if `schemars` is enabled)
- `pub struct FnNameTool; impl crate::tools::traits::Tool for FnNameTool { ... }`

Errors and diagnostics
- Missing `description`: compile-time error.
- `params` contains unknown parameter name: compile-time error telling which parameter is missing.
- Unsupported function parameter type (borrowed `&str`, lifetime): compile-time error recommending `String`.

Implementation plan
1. Create a `proc-macro` crate `mini_langchain_macros`.
2. Implement attribute parsing with `syn` and `quote`.
3. Generate the `Params` struct and wrapper `Tool` impl as described.
4. Add trybuild tests for success and common failure cases.

Usage
- Add `mini_langchain_macros` as a path dependency in the main crate and `use mini_langchain_macros::tool;` or re-export it for convenient usage.

This file is a design spec and minimal reference for the proc-macro implementation.
