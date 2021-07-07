use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::OnceCell;
use tracing::warn;
use tracing::{info, instrument};

use crate::languages::LanguagePool;
use crate::languages::Snippet;

pub static CMD_RGX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^~run *```([a-z]*)\n((?s).*)\n```").unwrap());

pub static LANGS_PATH: Lazy<String> =
    Lazy::new(|| env::var("SCRIPTY").expect("Environment variable: SCRIPTY_LANGS not set"));
pub static LANG_POOL: OnceCell<LanguagePool> = OnceCell::const_new();

#[instrument]
pub async fn create_docker_executors() {
    let mut languages_dir = fs::read_dir(&*LANGS_PATH)
        .await
        .expect("Could not find languages directory");
    while let Some(sub_dir) = languages_dir.next_entry().await.unwrap() {
        if !sub_dir.file_type().await.unwrap().is_dir() {
            continue;
        }
        let lang = sub_dir.file_name().into_string().unwrap();
        let executor = format!("{}_executor", lang);
        info!("Found docker configuration for {}", lang);

        let build = Command::new("docker")
            .arg("build")
            .arg(sub_dir.path())
            .arg("-t")
            .arg(format!("{}_executor", lang))
            .output()
            .await
            .unwrap();
        // TODO: redundant check with language specific tests
        if build.stderr.is_empty() {
            println!("{}", String::from_utf8(build.stdout).unwrap()); // FIXME: debug
            info!("{} created with success", executor);
        } else {
            println!("{}", String::from_utf8(build.stderr).unwrap()); // FIXME: debug
            warn!("{} failed to create", executor);
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

    // TODO: clean this code up
    // TODO: make it handle errors properly
    #[tokio::test]
    async fn test_executors() {
        create_docker_executors().await;
        let mut languages_dir = fs::read_dir(&*LANGS_PATH)
            .await
            .expect("Could not find languages directory");
        while let Some(sub_dir) = languages_dir.next_entry().await.unwrap() {
            if !sub_dir.file_type().await.unwrap().is_dir() {
                continue;
            }
            let lang = String::from(sub_dir.file_name().to_str().unwrap());
            println!("{}", lang);
            println!("{:?}", sub_dir.path());

            let mut lang_dir = fs::read_dir(sub_dir.path())
                .await
                .expect("Error opening language directory");
            while let Some(file) = lang_dir.next_entry().await.unwrap() {
                if file.file_type().await.unwrap().is_file() {
                    if file.file_name().to_str().unwrap().starts_with("test") {
                        let test_path = format!(
                            "{}/{}",
                            sub_dir.path().into_os_string().into_string().unwrap(),
                            file.file_name().to_str().unwrap()
                        );
                        let test_code =
                            String::from_utf8(fs::read(test_path).await.unwrap()).unwrap();

                        let executor = format!("{}_executor", lang);

                        println!("Code: {}", test_code);

                        let test_output = Snippet::new(executor, test_code)
                            .await
                            .run()
                            .await
                            .unwrap()
                            .output;

                        assert_eq!(format!("{}_test", lang), test_output.trim());
                    }
                }
            }
        }
    }
}
