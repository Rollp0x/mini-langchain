pub mod llm;
pub mod tools;
pub mod agent;
pub mod message;
pub mod config;
pub mod error;
pub mod prelude;

// re-export the proc-macro attribute for convenient use: `use mini_langchain::tool;` or `#[mini_langchain::tool(...)]`
#[allow(unused_imports)]
pub use mini_langchain_macros::tool;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
