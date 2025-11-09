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
    // let ollama = Ollama::default().with_model("qwen3:8b");
    let ollama = Ollama::default().with_model("deepseek-r1:latest");
    let llm: Arc<dyn mini_langchain::llm::traits::LLM> = Arc::new(ollama);

    let mut agent = Agent::new("Ollama_deepseek-r1:latest", llm, Some(5));

    // register echo tool
    agent.register_tool(None, Arc::new(GetWeatherTool));


    agent.set_system_prompt(
        r##"你是一天气预报智能助手,你可以查询工具或者直接回答"##);

    // Ask the model to get weather for Beijing so it should request the `get_weather` tool.
    let prompt = "What's the weather in Beijing?";

    match agent.call_llm(prompt).await {
        Ok(res) => {
            println!("generation: {:?}", res);
        }
        Err(e) => eprintln!("LLM error: {:?}", e),
    }
}
