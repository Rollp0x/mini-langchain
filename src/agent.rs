use std::collections::HashMap;
use std::sync::Arc;
use crate::llm::traits::LLM;
use crate::message::Message;
use crate::tools::{
    traits::Tool,
    schema::ToolSchema,
};
use serde_json::json;


pub mod types;
pub mod error;
pub mod traits;

use traits::AgentRunner;
use types::{Agent,AgentResult,AgentExecuteResult};
use error::AgentError;


impl Agent {
    /// Create a new Agent with the provided name and LLM. Tools start empty.
    pub fn new(name: impl Into<String>, llm: Arc<dyn LLM>,max_iterations:Option<usize>) -> Self {
        Self {
            name: name.into(),
            llm,
            tools: HashMap::new(),
            memory: Vec::new(),
            system_prompt: None,
            max_iterations: max_iterations.unwrap_or(100) ,
        }
    }

    /// Register a tool under the given name. Replaces any existing tool with the same name. Returns &mut Self for chaining.
    pub fn register_tool(&mut self, name: Option<&str>, tool: Arc<dyn Tool>) -> &mut Self {
        // If no name is provided, use the tool's own name.
        let name = name.unwrap_or_else(|| tool.name());
        self.tools.insert(name.into(), tool);
        self
    }

    /// Change the maximum iterations for the agent's decision process.
    pub fn change_max_iterations(&mut self, max_iterations: usize) {
        self.max_iterations = max_iterations;
    }   

    /// Look up a tool by name.
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Set or replace the agent's system prompt.
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    // generate system prompt
    pub fn generate_system_prompt(&self) -> Vec<Message> {
        let mut msgs = Vec::new();
        if let Some(prompt) = self.system_prompt.as_ref() {
            msgs.push(Message::system(prompt.clone()));
        }
        if !self.tools.is_empty() {
        msgs.push(Message::developer(
            format!("I also provide some tools for you to choose from. If you want to call a tool, please include the following JSON format in your response: {}

            IMPORTANT: After you have completed the task by calling all necessary tools, you MUST return a final response WITHOUT any tool_calls. Simply provide a summary or confirmation message to indicate completion. Do NOT continue calling tools after the task is done.", 
                json!({
                    "tool_calls": [
                        {
                            "name": "tool_name",
                            "args": {
                                "param1": "value1",
                                "param2": "value2"
                            }
                        }
                    ]
                }).to_string())
            ));
        }
        msgs
    }

    // 生成工具提示
    pub fn generate_tools_prompt(&self) -> Vec<Message> {
        self.tools.iter().map(|(name, tool)| {
            let schema = ToolSchema {
                name: name.clone(),
                description: tool.description().to_string(),
                args: tool.args(),
            };
            
            Message::system(serde_json::to_string(&schema).unwrap())
        }).collect()
    }
}



#[async_trait::async_trait]
impl AgentRunner for Agent {
    async fn call_llm(&self, prompt: &str) -> AgentExecuteResult {
        // Build a sequence of messages so LLM implementations that support
        // system/user roles can consume them properly.
        let mut msgs: Vec<Message> = self.generate_system_prompt();
        let tool_msgs = self.generate_tools_prompt();
        msgs.extend(tool_msgs);
        msgs.push(Message::user(prompt.to_string()));
        let mut result = AgentResult::default();
        let mut  counter:usize = 0;
        // Main loop: call LLM, check for tool calls, execute tools, repeat.
        while counter < self.max_iterations {
            // Call the LLM to get a response.
            let res = self.llm.generate(&msgs).await?;
            result.tokens.prompt_tokens += res.tokens.prompt_tokens;
            result.tokens.completion_tokens += res.tokens.completion_tokens;
            result.tokens.total_tokens += res.tokens.total_tokens;
            counter += 1;
            // check if there are tool calls
            if !res.tool_calls.is_empty() {
                // add assistant message
                msgs.push(Message::assistant(res.generation));
                // process tool calls
                for call_info in res.tool_calls {
                    let name = &call_info.name;
                    if let Some(tool_impl) = self.tools.get(name){
                        let tool_result = tool_impl.run(call_info.args).await?;
                        let tool_res_msg = Message::tool_res(
                            name,
                            format!("Tool {} returned: {}", name, tool_result));
                        msgs.push(tool_res_msg);
                    }else{
                        return Err(AgentError::ToolNotFound(call_info.name));
                    }
                }
            } else {
                // update generation
                result.generation = res.generation;
                return Ok(result);
            }
        }
        Err(AgentError::MaxIterationsExceeded(self.max_iterations))
    }
}