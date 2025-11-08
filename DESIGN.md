# Mini-LangChain Design Document

> 极简的 Rust LangChain 实现 - 专注核心功能，只支持文本输入

## 项目目标

### 核心理念
- **极简主义**：只实现必要功能，避免过度工程
- **类型安全**：充分利用 Rust 的类型系统
- **零成本抽象**：性能优先，避免不必要的运行时开销
- **自用优先**：为个人项目设计，不追求通用性

### 功能范围

✅ **支持的功能**
- 文本输入/输出（仅文本，不支持图片、音频等）
- 多 LLM 支持（OpenAI、Anthropic、Qwen、Deepseek、Ollama）
- Tool/Function Calling
- 简单的 Agent 模式（ReAct）
- 配置文件驱动

❌ **不支持的功能**
- 多模态（图片、音频、视频）
- 复杂的 Chain（只实现基础的）
- Memory（简化版或不实现）
- 向量数据库集成（后期可选）
- Document Loaders（后期可选）

---

## 架构设计

### 核心模块

```
mini-langchain/
├── src/
│   ├── lib.rs                  # 库入口
│   ├── llm/                    # LLM 抽象和实现
│   │   ├── mod.rs
│   │   ├── traits.rs           # LLM trait 定义
│   │   ├── openai.rs           # OpenAI 实现
│   │   ├── anthropic.rs        # Anthropic 实现
│   │   ├── qwen.rs             # Qwen 实现
│   │   ├── deepseek.rs         # Deepseek 实现
│   │   └── ollama.rs           # Ollama 实现
│   ├── tools/                  # Tool 系统
│   │   ├── mod.rs
│   │   ├── schema.rs           # ToolSchema（统一表示）
│   │   ├── macros.rs           # define_tool! 宏
│   │   └── builtin/            # 内置工具
│   │       ├── mod.rs
│   │       ├── calculator.rs
│   │       └── search.rs
│   ├── agent/                  # Agent 实现
│   │   ├── mod.rs
│   │   ├── simple.rs           # 简单的 Agent 循环
│   │   └── react.rs            # ReAct Agent
│   ├── message.rs              # 消息类型
│   ├── config.rs               # 配置管理
│   ├── error.rs                # 错误类型
│   └── prelude.rs              # 便捷导入
├── examples/                   # 示例代码
│   ├── simple_chat.rs
│   ├── tool_calling.rs
│   └── react_agent.rs
├── tests/                      # 集成测试
│   └── integration_test.rs
├── Cargo.toml
├── DESIGN.md                   # 本文档
└── README.md
```


---

## 核心类型设计

### 1. LLM Trait

```rust
/// 核心 LLM trait（极简版）
#[async_trait]
pub trait LLM: Send + Sync {
    /// 生成文本响应
    async fn generate(&self, messages: &[Message]) -> Result<String, LLMError>;
    
    /// 带工具的生成（可选）
    async fn generate_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<GenerateResult, LLMError> {
        // 默认实现：不支持工具
        Err(LLMError::ToolsNotSupported)
    }
    
    /// 流式生成（可选）
    async fn stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = String> + Send>>, LLMError> {
        // 默认实现：不支持流式
        Err(LLMError::StreamNotSupported)
    }
}
```

### 2. Message 类型

```rust
/// 消息类型（极简）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,  // 用于工具调用的名称
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            name: None,
        }
    }
    
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            name: None,
        }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            name: None,
        }
    }
    
    pub fn tool(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            name: Some(name.into()),
        }
    }
}
```

### 3. Tool 系统

```rust
/// 统一的 Tool Schema（中间表示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: Option<String>,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 生成结果（可能包含工具调用）
#[derive(Debug, Clone)]
pub struct GenerateResult {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Tool trait
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    async fn run(&self, input: serde_json::Value) -> Result<String, ToolError>;
}

/// 定义工具的宏（简化版示例）
define_tool! {
    struct CalculatorTool {
        name: "calculator",
        description: "执行数学计算",
        parameters: {
            expression: String {
                description: "数学表达式，例如: 2+2 或 5*10",
                required: true,
            }
        },
        run: |input| async move {
            let expr = input["expression"].as_str()
                .ok_or_else(|| ToolError::InvalidInput("缺少表达式".into()))?;
            let result = eval_expression(expr)?;
            Ok(result.to_string())
        }
    }
}
```

### 4. LLM 特定格式转换
```rust
// 为每个 LLM 的工具格式实现 From
impl From<&ToolSchema> for OpenAIToolSchema {
    fn from(schema: &ToolSchema) -> Self { ... }
}

impl From<&ToolSchema> for AnthropicToolSchema {
    fn from(schema: &ToolSchema) -> Self { ... }
}
```

### 5. Agent
```rust
/// 简单的 Agent
pub struct SimpleAgent {
    llm: Box<dyn LLM>,
    tools: Vec<Arc<dyn Tool>>,
    max_iterations: usize,
}

impl SimpleAgent {
    pub async fn run(&self, task: &str) -> Result<String, AgentError> {
        // ReAct 循环：Think → Act → Observe
        for _ in 0..self.max_iterations {
            // 1. LLM 思考
            let response = self.llm.generate_with_tools(...).await?;
            
            // 2. 解析工具调用
            if let Some(tool_call) = parse_tool_call(&response) {
                // 3. 执行工具
                let result = execute_tool(&tool_call).await?;
                // 4. 继续循环
                continue;
            }
            
            // 5. 返回最终答案
            return Ok(response);
        }
        Err(AgentError::MaxIterationsReached)
    }
}
```


---

## 配置系统

### 配置文件格式 (config.toml)


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

# 可选：工具特定配置
[tools.search]
api_key = "search-api-key"
max_results = 5
```

### 配置加载


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
        // 从环境变量读取，优先级：
        // MINI_LANGCHAIN_PROVIDER -> llm.provider
        // MINI_LANGCHAIN_API_KEY -> llm.api_key
        // MINI_LANGCHAIN_MODEL -> llm.model
        unimplemented!("从环境变量加载配置")
    }
}
```

---

## 错误处理

### 统一的错误类型


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

## 依赖管理


```toml
[package]
name = "mini-langchain"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "Minimal Rust LangChain implementation for text-only interactions"
license = "MIT OR Apache-2.0"

[dependencies]
# 异步运行时
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "stream"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# 错误处理
thiserror = "1"
anyhow = "1"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tokio-test = "0.4"
mockito = "1"
```

---

## 实现计划

### Phase 1: 核心基础 (Week 1)
- [ ] 项目结构搭建
- [ ] Message、Error 等基础类型
- [ ] LLM trait 定义
- [ ] OpenAI 基础实现（只实现 generate）
- [ ] 配置系统

### Phase 2: Tool 系统 (Week 2)
- [ ] ToolSchema 设计
- [ ] Tool trait 定义
- [ ] define_tool! 宏实现
- [ ] 内置工具：Calculator
- [ ] OpenAI Function Calling 集成

### Phase 3: 多 LLM 支持 (Week 3)
- [ ] Anthropic 实现
- [ ] Qwen 实现
- [ ] Deepseek 实现
- [ ] Ollama 实现
- [ ] From<&ToolSchema> 适配器

### Phase 4: Agent (Week 4)
- [ ] SimpleAgent 实现
- [ ] ReAct 模式
- [ ] 工具执行循环
- [ ] 错误处理和重试

### Phase 5: 优化和示例 (Week 5)
- [ ] 流式输出支持
- [ ] 完整的示例代码
- [ ] 文档和注释
- [ ] 单元测试

---

## 使用示例

### 示例 1: 简单对话


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

### 示例 2: 使用工具


```rust
use mini_langchain::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_file("config.toml")?;
    let llm = create_llm(&config.llm)?;
    
    // 定义工具
    let calculator = Arc::new(CalculatorTool);
    
    // 准备工具 schema
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

### 示例 3: Agent


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

---

## 设计原则

### 1. 简单优于复杂

- 只实现必要功能
- 避免过度抽象
- 代码行数控制在 2000 行以内

### 2. 类型安全优先

- 充分利用 Rust 类型系统
- 编译时捕获错误
- 避免 unwrap()，使用 Result

### 3. 零成本抽象

- 使用编译时泛型而非运行时多态（当适用时）
- 避免不必要的堆分配
- 性能敏感部分使用 `#[inline]`

### 4. 实用主义

- 优先使用标准库（`From`, `Into`, `Error` 等）
- 不重复造轮子（使用成熟的 crate）
- 能用 `serde_json::Value` 就不定义新类型

### 5. 可扩展性

- 为将来扩展留接口
- 但不为假想的需求编码

---

## 后期可选功能

### 优先级 2
- [ ] 流式输出完整支持
- [ ] 简单的对话历史管理
- [ ] 更多内置工具（天气查询、文件操作等）
- [ ] 重试机制和错误恢复

### 优先级 3
- [ ] 基础的 Document Loader
- [ ] 简单的向量存储（基于 FAISS 或 Qdrant）
- [ ] RAG 支持
- [ ] Prompt 模板系统

---

## 贡献指南


这是一个自用项目，但欢迎：

- 🐛 Bug 修复
- 📝 文档改进
- 💡 简单功能建议

**不欢迎：**

- ❌ 复杂功能
- ❌ 过度抽象
- ❌ 破坏简洁性的 PR

---

## 许可证

MIT OR Apache-2.0

---

## 参考资料

- [LangChain Python](https://github.com/langchain-ai/langchain)
- [LangChain.js](https://github.com/langchain-ai/langchainjs)
- [langchain-rust](https://github.com/Abraxas-365/langchain-rust)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference)
- [Anthropic API Reference](https://docs.anthropic.com/claude/reference)

---

## 常见问题

### Q: 为什么不支持多模态？
A: 为了保持简单，专注于文本处理。多模态会增加很多复杂性。

### Q: 为什么不使用 trait object 而是泛型？
A: 在可能的地方使用泛型以获得零成本抽象，但对于 LLM 和 Tool 等需要动态分发的场景，仍使用 trait object。

### Q: 如何添加新的 LLM 提供商？
A: 实现 `LLM` trait 即可。可以参考 `openai.rs` 的实现。

### Q: 性能如何？
A: 由于是网络 IO 密集型应用，性能瓶颈主要在 API 调用上。Rust 实现本身几乎没有额外开销。

