

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Missing configuration: {0}")]
    MissingConfig(String),
}