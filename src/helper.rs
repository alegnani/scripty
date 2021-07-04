use once_cell::sync::Lazy;
use regex::Regex;
use tokio::process::Command;
use tracing::warn;
use std::env;
use tracing::{info, instrument};
use tokio::sync::OnceCell;
use tokio::fs;

use crate::languages::LanguagePool;

pub static CMD_RGX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^~run *```([a-z]*)\n((?s).*)\n```").unwrap());

pub static LANGS_PATH: Lazy<String> =
    Lazy::new(|| env::var("SCRIPTY").expect("Environment variable: SCRIPTY_LANGS not set"));
pub static LANG_POOL: OnceCell<LanguagePool> = OnceCell::const_new();

#[instrument]
pub async fn create_docker_executors() {
    let mut languages_dir = fs::read_dir(&*LANGS_PATH).await.expect("Could not find languages directory");
    while let Some(sub_dir) = languages_dir.next_entry().await.unwrap() {
        if !sub_dir.file_type().await.unwrap().is_dir() {
            continue;
        }
        let lang = sub_dir.file_name().into_string().unwrap();
        info!("Found docker configuration for {}", lang);
        let build = Command::new("docker").arg("build").arg(sub_dir.path()).arg("-t").arg(format!("{}_executor", lang)).output().await.unwrap();
        // TODO: redundant check with language specific tests
        if build.stderr.is_empty() {
            println!("{}", String::from_utf8(build.stdout).unwrap()); // FIXME: debug
            info!("{}_executor created with success", lang);
        } else {
            println!("{}", String::from_utf8(build.stderr).unwrap()); // FIXME: debug
            warn!("{}_executor failed to create", lang);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_helper() {
        println!("{:?}", CMD_RGX);
        println!("{}", *LANGS_PATH);
        let _ = LANG_POOL.set(LanguagePool::new().await);
        create_docker_executors().await;
    }
}
