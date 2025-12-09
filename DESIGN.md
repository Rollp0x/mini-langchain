
# Mini-LangChain Design Document

> Minimal Rust LangChain implementation - focus on core features, text-only, type-safe, and easy to use.

## Project Goals

### Core Principles
- **Minimalism**: Only essential features, avoid over-engineering
- **Type Safety**: Leverage Rust's type system
- **Zero-cost Abstraction**: Performance first, avoid unnecessary runtime overhead
- **Personal Use First**: Designed for personal/small projects

### Supported Features

- Text input/output (text only)
- Multiple LLM support (OpenAI, Anthropic, Qwen, Deepseek, Ollama)
- Tool/Function Calling
- Agent mode (ReAct)
- Config file driven

### Not Supported

- Multimodal (image/audio/video)
- Complex chains (only basic supported)
- Vector DB/document loaders (future optional)

---

## Architecture Overview

### Main Modules

```
mini-langchain/
├── src/
│   ├── lib.rs            # Library entry point
│   ├── llm/              # LLM abstraction and implementations
│   ├── tools/            # Tool system
│   ├── agent/            # Agent implementation
│   ├── message.rs        # Message types
│   ├── config.rs         # Config management
│   ├── error.rs          # Error types
│   └── prelude.rs        # Convenient imports
├── Cargo.toml
├── DESIGN.md
└── README.md
```


## Core Type Design

### LLM Trait

```rust
// src/llm/traits.rs (simplified)
#[async_trait]
pub trait LLM: Send + Sync {
    async fn generate(&self, messages: &[Message]) -> LLMResult<GenerateResult>;
    // ...
}
```

### Message Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    ToolResponce,
    Tool,
    Developer,
}
```

### Tool System


```rust
// src/tools/schema.rs (simplified)
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    async fn run(&self, input: serde_json::Value) -> Result<String, ToolError>;
}

// You can define custom tools using the proc-macro attribute:
#[tool(
    name = "get_weather",
    description = "Get weather for a given city",
    params(city = "City name, e.g. 'San Francisco'")
)]
fn get_weather(city: String) -> String {
    format!("It's always sunny in {}!", city)
}
```

### Agent

```rust
// src/agent/types.rs (simplified)
pub struct Agent {
    pub name: String,
    pub llm: Arc<dyn LLM>,
    pub tools: HashMap<String, Arc<dyn Tool>>,
    pub memory: Vec<Message>,
    pub system_prompt: Option<String>,
    pub max_iterations: usize,
}

impl Agent {
    pub fn new(name: impl Into<String>, llm: Arc<dyn LLM>, max_iterations: Option<usize>) -> Self { ... }
    pub fn register_tool(&mut self, name: Option<&str>, tool: Arc<dyn Tool>) -> &mut Self { ... }
    pub fn run_task(&mut self, task: &str) -> ... { ... }
}
```
```


---

## Config

### config.toml


```toml
[llm]
provider = "openai"     # openai | anthropic | qwen | deepseek | ollama
model = "gpt-4"
api_key = "sk-..."       # Optional, read from environment variables
base_url = "https://..." # Optional

[agent]
max_iterations = 5
temperature = 0.7

[tools]
enabled = ["calculator", "search"]

# Optional: Tool-specific configurations
[tools.search]
api_key = "search-api-key"
max_results = 5
```

### Configuration Loading


```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub llm: LLMConfig,
    pub agent: AgentConfig,
    pub tools: ToolsConfig,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(Into::into)
    }
    
    pub fn from_env() -> Result<Self, ConfigError> {
        // MINI_LANGCHAIN_PROVIDER -> llm.provider
        // MINI_LANGCHAIN_API_KEY -> llm.api_key
        // MINI_LANGCHAIN_MODEL -> llm.model
        unimplemented!("Load configuration from environment variables")
    }
}
```

---

## Error Handling

### Unified Error Types


```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),
    
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---


## Implementation Plan

### Phase 1: Core Foundation (Week 1)
- [ ] Project structure setup
- [ ] Message and Error base types
- [ ] LLM trait definition
- [ ] Basic OpenAI implementation (only generate)
- [ ] Configuration system

### Phase 2: Tool System (Week 2)
- [ ] ToolSchema design
- [ ] Tool trait definition
- [ ] define_tool! macro implementation
- [ ] Built-in tool: Calculator
- [ ] OpenAI Function Calling integration

### Phase 3: Multi-LLM Support (Week 3)
- [ ] Anthropic implementation
- [ ] Qwen implementation
- [ ] Deepseek implementation
- [ ] Ollama implementation
- [ ] From<&ToolSchema> adapter

### Phase 4: Agent (Week 4)
- [ ] SimpleAgent implementation
- [ ] ReAct mode
- [ ] Tool execution loop
- [ ] Error handling and retry

### Phase 5: Optimization & Examples (Week 5)
- [ ] Streaming output support

## Error Handling

### Unified Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## Example Usage


### Simple Chat (Config-based)

```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file("config.toml")?;
    let llm = create_llm(&config.llm)?;
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("What is Rust?"),
    ];
    let response = llm.generate(&messages).await?;
    println!("{}", response);
    Ok(())
}
```

### Ollama Direct Usage Example

```rust
use mini_langchain::llm::ollama::Ollama;
use mini_langchain::message::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Ollama wrapper and select a local model (e.g. qwen3:8b)
    let ollama = Ollama::default().with_model("qwen3:8b");

    let messages = vec![Message::user("Why is the sky blue?")];

    match ollama.generate(&messages).await {
        Ok(res) => {
            println!("generation: {}", res.generation);
            let tokens = res.tokens;
            println!("tokens: prompt={} completion={} total={}", tokens.prompt_tokens, tokens.completion_tokens, tokens.total_tokens);
        }
        Err(e) => eprintln!("error calling Ollama: {:?}", e),
    }
    Ok(())
}
```

### Agent Example (Config-based)

```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file("config.toml")?;
    let llm = create_llm(&config.llm)?;
    let tools = vec![
        Arc::new(CalculatorTool) as Arc<dyn Tool>,
        Arc::new(SearchTool) as Arc<dyn Tool>,
    ];
    let mut agent = Agent::new("assistant", llm, Some(5));
    for tool in &tools {
        agent.register_tool(None, tool.clone());
    }
    let result = agent.run_task("What's the weather in Beijing today? If the temperature is above 25°C, calculate 25 * 1.8 + 32.").await?;
    println!("Result: {}", result);
    Ok(())
}
```

### Agent Example with Ollama and Custom Tool

```rust
use mini_langchain::{
    *,
    llm::ollama::Ollama,
    agent::{
        types::Agent,
        traits::AgentRunner
    }
};
use std::sync::Arc;

// Use the proc-macro attribute to generate the Tool implementation
#[tool(
    name = "get_weather",
    description = "Get weather for a given city",
    params(city = "City name, e.g. 'San Francisco'")
)]
fn get_weather(city: String) -> String {
    format!("It's always sunny in {}!", city)
}

#[tokio::main]
async fn main() {
    // Adjust model name to one available in your Ollama server.
    let ollama = Ollama::default().with_model("qwen3:8b");
    let llm: Arc<dyn mini_langchain::llm::traits::LLM> = Arc::new(ollama);

    let mut agent = Agent::new("Ollama_qwen3:8b", llm, Some(5));
    agent.register_tool(None, Arc::new(GetWeatherTool));

    agent.set_system_prompt(
        r##"You are a weather forecasting intelligent assistant. You can query tools or answer directly."##);

    let prompt = "What's the weather in Beijing?";

    match agent.call_llm(prompt).await {
        Ok(res) => {
            println!("generation: {:?}", res);
        }
        Err(e) => eprintln!("LLM error: {:?}", e),
    }
}
```

## Design Principles

1. **Simplicity over complexity**: Only implement what is needed, avoid over-abstraction, keep codebase small.
2. **Type safety**: Leverage Rust's type system, prefer Result over unwrap().
3. **Zero-cost abstraction**: Prefer generics over trait objects when possible, avoid unnecessary heap allocations.
4. **Pragmatism**: Use standard crates, don't reinvent the wheel, use serde_json::Value for flexibility.
5. **Extensibility**: Leave room for future extension, but don't code for imaginary needs.

---

## License

MIT OR Apache-2.0

## References

- [LangChain Python](https://github.com/langchain-ai/langchain)
- [LangChain.js](https://github.com/langchain-ai/langchainjs)
- [langchain-rust](https://github.com/Abraxas-365/langchain-rust)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference)
- [Anthropic API Reference](https://docs.anthropic.com/claude/reference)
