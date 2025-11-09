
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,           // 系统指令
    User,             // 用户输入
    Assistant,        // AI回复
    ToolResponce,     // Tool执行返回
    Tool,             //用户预定义函数
    Developer,       //开发者消息,兼容openai
}

/// 消息类型（极简）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,  // 用于工具调用的名称
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
    pub fn tool_res(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            name: Some(name.into()),
        }
    }

    pub fn developer(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Developer,
            content: content.into(),
            name: None,
        }
    }   
}