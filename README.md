# Mini-LangChain

> 极简的 Rust LangChain 实现 - 专注核心功能，只支持文本输入

[![Crates.io](https://img.shields.io/crates/v/mini-langchain.svg)](https://crates.io/crates/mini-langchain)
[![Documentation](https://docs.rs/mini-langchain/badge.svg)](https://docs.rs/mini-langchain)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 特性

- 🦀 **纯 Rust 实现** - 类型安全，零成本抽象
- 🤖 **多 LLM 支持** - OpenAI、Anthropic、Qwen、Deepseek、Ollama
- 🛠️ **Tool Calling** - 函数调用和工具集成（需要 LLM 支持，部分 LLM 不支持）
- 🤖 **Agent 模式** - 支持 ReAct 模式的智能代理
- 📝 **仅文本** - 专注文本处理，保持简单
- ⚙️ **配置驱动** - TOML 配置文件支持

## 快速开始

### 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
mini-langchain = "0.1"
tokio = { version = "1", features = ["full"] }
```

### 简单对话

```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 从配置文件加载
    let config = Config::from_file("config.toml")?;
    let llm = create_llm(&config.llm)?;
    
    // 简单对话
    let messages = vec![
        Message::system("你是一个有帮助的助手"),
        Message::user("什么是 Rust？"),
    ];
    
    let response = llm.generate(&messages).await?;
    println!("{}", response);
    
    Ok(())
}
```

### 使用工具

```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file("config.toml")?;
    let llm = create_llm(&config.llm)?;
    
    // 定义工具
    let calculator = Arc::new(CalculatorTool);
    let tools = vec![calculator.schema()];
    
    let messages = vec![
        Message::user("计算 25 * 4 等于多少？"),
    ];
    
    let result = llm.generate_with_tools(&messages, &tools).await?;
    
    if let Some(tool_calls) = result.tool_calls {
        for call in tool_calls {
            let output = calculator.run(call.arguments).await?;
            println!("工具结果: {}", output);
        }
    }
    
    Ok(())
}
```

### Agent 示例

```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file("config.toml")?;
    let llm = create_llm(&config.llm)?;
    
    // 创建工具
    let tools = vec![
        Arc::new(CalculatorTool) as Arc<dyn Tool>,
        Arc::new(SearchTool) as Arc<dyn Tool>,
    ];
    
    // 创建 Agent
    let agent = SimpleAgent::new(llm, tools);
    
    // 运行任务
    let result = agent.run(
        "北京今天天气怎么样？如果温度超过 25 度，计算 25 * 1.8 + 32"
    ).await?;
    
    println!("结果: {}", result);
    
    Ok(())
}
```

## 配置

创建 `config.toml` 文件：

```toml
[llm]
provider = "openai"     # openai | anthropic | qwen | deepseek | ollama
model = "gpt-4"
api_key = "sk-..."      # 可选，从环境变量读取
base_url = "https://..." # 可选

[agent]
max_iterations = 5
temperature = 0.7

[tools]
enabled = ["calculator", "search"]
```

## 支持的 LLM

| Provider | 状态 | 流式输出 | Function Calling |
|----------|------|---------|------------------|
| OpenAI | ✅ | ✅ | ✅ |
| Anthropic | 🚧 | 🚧 | 🚧 |
| Qwen | 🚧 | 🚧 | 🚧 |
| Deepseek | 🚧 | 🚧 | 🚧 |
| Ollama | 🚧 | 🚧 | 🚧 |

## 内置工具

- `CalculatorTool` - 数学计算
- `SearchTool` - 网络搜索（需要配置 API）

## 文档

详细的设计文档请查看 [DESIGN.md](DESIGN.md)

## 为什么是 Mini？

与 [langchain-rust](https://github.com/Abraxas-365/langchain-rust) 不同，`mini-langchain` 专注于：

- ✅ **极简主义** - 只实现必要功能
- ✅ **自用优先** - 为个人项目设计
- ✅ **易于理解** - 代码行数 < 2000
- ❌ **不追求通用性** - 只支持文本
- ❌ **不支持所有功能** - 按需实现

## 开发状态

🚧 **Alpha 阶段** - API 可能会有变动

当前进度：
- [x] 项目架构设计
- [ ] 核心 LLM trait
- [ ] OpenAI 实现
- [ ] Tool 系统
- [ ] Agent 实现
- [ ] 文档和测试

## 贡献

欢迎贡献！但请保持简洁：

- 🐛 Bug 修复
- 📝 文档改进
- 💡 简单功能建议

**不欢迎：** 复杂功能、过度抽象、破坏简洁性的 PR

## 许可证

MIT OR Apache-2.0

## 参考

- [LangChain Python](https://github.com/langchain-ai/langchain)
- [LangChain.js](https://github.com/langchain-ai/langchainjs)
- [langchain-rust](https://github.com/Abraxas-365/langchain-rust)
