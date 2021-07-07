use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::OnceCell;
use tracing::{error, info, instrument};

use crate::languages::{Executable, LanguagePool, Response};

pub static CMD_RGX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^run *```([a-z]*)\n((?s).*)\n```").unwrap());

pub static LANGS_PATH: Lazy<String> =
    Lazy::new(|| env::var("SCRIPTY").expect("Environment variable: SCRIPTY_LANGS not set"));
pub static LANG_POOL: OnceCell<LanguagePool> = OnceCell::const_new();

#[instrument]
pub async fn create_docker_executors() -> Result<()> {
    // open ../scripty_bot/languages directory
    let mut languages_dir = fs::read_dir(&*LANGS_PATH)
        .await
        .expect("Could not find languages directory");
    // iterate over all of its subdirectories, which represent different languages
    while let Some(language) = languages_dir.next_entry().await? {
        if language.file_type().await?.is_file() {
            continue;
        }
        let lang = language.file_name().into_string().unwrap();
        // build the docker image for every different language
        let _ = Command::new("docker")
            .arg("build")
            .arg(language.path())
            .arg("-t")
            .arg(format!("{}_executor", lang))
            .output()
            .await?;
        info!("Built docker image for {}", lang);
        // use the test.* file to check if the language works
        check_executor(language.path(), &lang).await?;

        info!("Docker image for {} passed all tests", lang);
    }
    Ok(())
}

#[instrument]
async fn check_executor(language_dir_path: PathBuf, lang: &str) -> Result<()> {
    let mut language_dir = fs::read_dir(&language_dir_path).await?;
    // iterate over all of the files in the directory of a language
    while let Some(file) = language_dir.next_entry().await? {
        // find the test.* file
        if file.file_type().await?.is_file() {
            let file_name = file
                .file_name()
                .to_str()
                .ok_or_else(|| anyhow!("Could not get file name"))?
                .to_string();

            if file_name.starts_with("test") {
                let test_file_path = format!(
                    "{}/{}",
                    language_dir_path
                        .to_str()
                        .ok_or_else(|| anyhow!("Could not get dir name"))?,
                    file_name
                );
                // read the test.* file
                let test_code = String::from_utf8(fs::read(test_file_path).await?)?;
                let executor = format!("{}_executor", lang);
                // run the code in the test.* file
                if let Response::Output(res, _exec_time) = Executable::new(executor, test_code)
                    .await
                    .run()
                    .await
                    .unwrap()
                {
                    // check that the result is correct and the executor is working
                    let test_output = res.trim();
                    if format!("{}_test", lang) != test_output {
                        error!("Expected: {}_test, Got: {}", lang, test_output);
                        return Err(anyhow!("Executor did not pass test"));
                    }
                } else {
                error!("{}_executor timed out", lang);
                return Err(anyhow!("Executor timed out"));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_helper() {
        println!("{:?}", CMD_RGX);
        println!("{}", *LANGS_PATH);
        let _ = LANG_POOL.set(LanguagePool::new().await);
    }

    #[tokio::test]
    async fn test_executors() {
        create_docker_executors().await.unwrap();
    }
}
