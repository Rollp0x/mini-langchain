enabled = ["calculator", "search"]

# Mini-LangChain

> Minimal Rust LangChain implementation - focus on core features, text-only, type-safe, and easy to use.

[![Crates.io](https://img.shields.io/crates/v/mini-langchain.svg)](https://crates.io/crates/mini-langchain)
[![Documentation](https://docs.rs/mini-langchain/badge.svg)](https://docs.rs/mini-langchain)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- 🦀 **Pure Rust** - Type safety, zero-cost abstraction
- 🤖 **Multiple LLMs** - OpenAI, Anthropic, Qwen, Deepseek, Ollama
- 🛠️ **Tool Calling** - Function/tool integration (if supported by LLM)
- 🤖 **Agent Mode** - ReAct-style agent loop
- 📝 **Text Only** - Focused on text processing
- ⚙️ **Config Driven** - TOML config file

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mini-langchain = "0.1"
tokio = { version = "1", features = ["full"] }
```


### Simple Chat (Config-based)

```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load from config file
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

### Tool Usage

```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file("config.toml")?;
    let llm = create_llm(&config.llm)?;

    // Define tool
    let calculator = Arc::new(CalculatorTool);
    let tools = vec![calculator.schema()];

    let messages = vec![
        Message::user("What is 25 * 4?")
    ];

    let result = llm.generate_with_tools(&messages, &tools).await?;

    if let Some(tool_calls) = result.call_tools {
        for call in tool_calls {
            let output = calculator.run(call.args).await?;
            println!("Tool result: {}", output);
        }
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

    // Create tools
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

## Configuration

Create a `config.toml` file:

```toml
[llm]
provider = "openai"     # openai | anthropic | qwen | deepseek | ollama
model = "gpt-4"
api_key = "sk-..."      # Optional, can be read from env
base_url = "https://..." # Optional

[agent]
max_iterations = 5
temperature = 0.7

[tools]

```

## Supported LLMs

| Provider   | Status | Streaming | Function Calling |
|------------|--------|-----------|------------------|
| Ollama     | ✅     | ✅        | ✅               |
| OpenAI     | ✅     | ✅        | ✅               |
| Anthropic  | 🚧     | 🚧        | 🚧               |
| Qwen       | 🚧     | 🚧        | 🚧               |
| Deepseek   | 🚧     | 🚧        | 🚧               |


## Built-in Tools & Custom Tools

- `CalculatorTool` - Math calculation
- `SearchTool` - Web search (requires API config)
- Custom tools can be defined using the `#[tool(...)]` proc-macro attribute

## Documentation

See [DESIGN.md](DESIGN.md) for detailed design.

## Why Mini?

Unlike [langchain-rust](https://github.com/Abraxas-365/langchain-rust), `mini-langchain` focuses on:

- ✅ **Minimalism** - Only essential features
- ✅ **Personal Use** - Designed for personal/small projects
- ✅ **Easy to Understand** - <2000 lines of code
- ❌ **Not General Purpose** - Text only
- ❌ **Not All Features** - Implement as needed

## Project Status

🚧 **Alpha** - API may change

## Contributing

Contributions are welcome! Please keep it simple:

- 🐛 Bug fixes
- 📝 Docs improvements
- 💡 Simple feature suggestions

**Not welcome:** Complex features, over-abstraction, or PRs that break simplicity

## License

MIT OR Apache-2.0

## References

- [LangChain Python](https://github.com/langchain-ai/langchain)
- [LangChain.js](https://github.com/langchain-ai/langchainjs)
- [langchain-rust](https://github.com/Abraxas-365/langchain-rust)
