use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use tokio::sync::OnceCell;

use crate::language_pool::LanguagePool;

pub static CMD_RGX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^~run *```([a-z]*)\n((?s).*)\n```").unwrap());

pub static SCRIPTS_PATH: Lazy<String> = Lazy::new(|| {
    env::var("SCRIPTY_SCRIPTS").expect("Environment variable: SCRIPTY_SCIPTS not set")
});
pub static LANGS_PATH: Lazy<String> =
    Lazy::new(|| env::var("SCRIPTY_LANGS").expect("Environment variable: SCRIPTY_LANGS not set"));
pub static LANG_POOL: OnceCell<LanguagePool> = OnceCell::const_new();

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_helper() {
        println!("{:?}", CMD_RGX);
        println!("{}", *SCRIPTS_PATH);
        println!("{}", *LANGS_PATH);
        let _ = LANG_POOL.set(LanguagePool::new().await);
    }
}
