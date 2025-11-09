use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgSchema {
    pub name: String,
    pub arg_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub args: Vec<ArgSchema>,
}
