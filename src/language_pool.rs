use crate::{helper::LANGS_PATH, language::Language};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use tokio::fs::{self, DirEntry};

pub struct LanguagePool {
    pool: HashMap<String, Language>,
}

impl LanguagePool {
    pub async fn new() -> Self {
        let mut pool = HashMap::new();
        let rgx = Regex::new(r"^([a-z]*)\n((?s).*)").unwrap();
        let mut lang_dir = fs::read_dir(&*LANGS_PATH).await.unwrap();
        while let Some(entry) = lang_dir.next_entry().await.unwrap() {
            if let Ok(lang) = Self::read_file(entry, &rgx).await {
                pool.insert(lang.get_name().await, lang);
            }
        }
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> Result<&Language> {
        self.pool
            .get(key)
            .ok_or_else(|| anyhow!("Language is not supported"))
    }

    async fn read_file(entry: DirEntry, rgx: &Regex) -> Result<Language> {
        let name = entry.file_name().into_string().unwrap();
        let path = entry.path();
        let contents = String::from_utf8(fs::read(path).await.unwrap()).unwrap();
        if !rgx.is_match(&contents) {
            return Err(anyhow!("File does not match RegEx"));
        }
        let captures = rgx.captures(&contents).unwrap();
        let extension = captures.get(1).unwrap().as_str().into();
        let launch = captures.get(2).unwrap().as_str().into();
        Ok(Language::new(name, extension, launch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create() {
        let pool = LanguagePool::new().await;
        println!("{:?}", pool.pool);
    }

    #[tokio::test]
    async fn get_language() {
        let pool = LanguagePool::new().await;
        let supp_lang = "python";
        let unsupp_lang = "kotlin";
        pool.get(supp_lang).await.unwrap();
        pool.get(unsupp_lang).await.unwrap_err();
    }
}
