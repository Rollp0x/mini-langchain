use super::error::ToolError;

// re-export ArgSchema for macros use
pub use super::schema::ArgSchema;

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn args(&self) -> Vec<ArgSchema>;
    async fn run(&self, input: serde_json::Value) -> Result<String, ToolError>;
}