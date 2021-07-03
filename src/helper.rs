use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use tokio::sync::OnceCell;

use crate::languages::LanguagePool;

pub static CMD_RGX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^~run *```([a-z]*)\n((?s).*)\n```").unwrap());

pub static LANGS_PATH: Lazy<String> =
    Lazy::new(|| env::var("SCRIPTY_LANGS").expect("Environment variable: SCRIPTY_LANGS not set"));
pub static LANG_POOL: OnceCell<LanguagePool> = OnceCell::const_new();

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_helper() {
        println!("{:?}", CMD_RGX);
        println!("{}", *LANGS_PATH);
        let _ = LANG_POOL.set(LanguagePool::new().await);
    }
}
