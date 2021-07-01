use anyhow::{anyhow, Result};
use regex::Regex;
use serenity::futures::io::Seek;
use uuid::Uuid;
use std::collections::HashMap;
use std::time;
use tokio::{
    fs::{self, DirEntry},
    io::AsyncWriteExt,
};

#[derive(Debug)]
pub struct Language {
    name: String,
    extension: String,
    launch: String,
}

impl Language {
    pub fn new(name: String, extension: String, launch: String) -> Self {
        Self {
            name,
            extension,
            launch,
        }
    }

    pub async fn run(&self, code: String) {}

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

struct Container {
    dir: String,
}

impl Container {
    pub async fn new() -> Result<Self> {
        let uuid = uuid::Uuid::new_v4();
        let dir = format!("scripts/task{}", &uuid);
        match fs::create_dir(&dir).await {
            Ok(_) => Ok(Self { dir }),
            Err(_) => Err(anyhow!("Could not create directory for container")),
        }
    }

    pub async fn create_file(&self, code: &str, extension: &str) {
        let path = format!("{}/main.{}", &self.dir, extension);
        let mut f = fs::File::create(path).await.unwrap();
        f.write_all(&code.as_bytes()).await.unwrap();
    }

    pub async fn clean_up(self) -> std::io::Result<()> {
        fs::remove_dir_all(&self.dir).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn container_alloc_dealloc() {
        let c = Container::new().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        c.clean_up().await.unwrap();
    }

    #[tokio::test]
    async fn container_add_file() {
        let c = Container::new().await.unwrap();
        c.create_file("print('bestia')", "py").await;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        c.clean_up().await.unwrap();
    }
}
