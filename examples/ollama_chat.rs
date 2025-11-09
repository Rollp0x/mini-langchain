use mini_langchain::llm::ollama::Ollama;
use mini_langchain::message::Message;
use mini_langchain::llm::traits::LLM;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create default Ollama wrapper and override model to a local one (e.g. qwen3:8b)
    let ollama = Ollama::default().with_model("qwen3:8b");

    // Simple user message
    let messages = vec![Message::user("为什么天空是蓝色？".to_string())];

    // Call generate (non-streaming)
    match ollama.generate(&messages).await {
        Ok(res) => {
            println!("generation: {}", res.generation);
            let tokens =  res.tokens;
            println!("tokens: prompt={} completion={} total={}", tokens.prompt_tokens, tokens.completion_tokens, tokens.total_tokens);
            
        }
        Err(e) => eprintln!("error calling Ollama: {:?}", e),
    }

    Ok(())
}
