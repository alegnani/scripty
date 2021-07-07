use anyhow::{anyhow, Result};
use std::{collections::HashSet, process::Stdio};
use tokio::{
    fs,
    io::AsyncWriteExt,
    process::Command,
    time::{Duration, Instant},
};
use tracing::{error, info, instrument, warn};

use crate::helper::{CMD_RGX, LANGS_PATH, LANG_POOL};

pub async fn run_pipeline(msg: &str) -> Result<Response> {
    let (lang_str, code) = parse(msg).await?;
    let snippet = LANG_POOL.get().unwrap().new_snippet(lang_str, code).await?;
    let res = snippet.run().await?;
    Ok(res)
}

#[instrument]
pub async fn parse(msg: &str) -> Result<(String, String)> {
    if !CMD_RGX.is_match(&msg) {
        error!("Message does not match regex");
        return Err(anyhow!("Regex does not match command"));
    }
    let captures = CMD_RGX.captures(msg).unwrap();
    let lang_str = captures.get(1).unwrap().as_str().into();
    let code = captures.get(2).unwrap().as_str().into();
    Ok((lang_str, code))
}

#[derive(Debug)]
pub struct LanguagePool {
    set: HashSet<String>,
}

impl LanguagePool {
    pub async fn new() -> LanguagePool {
        let mut set = HashSet::new();
        let mut lang_dir = fs::read_dir(&*LANGS_PATH).await.unwrap();
        while let Some(entry) = lang_dir.next_entry().await.unwrap() {
            if entry.file_type().await.unwrap().is_dir() {
                set.insert(entry.file_name().into_string().unwrap());
            }
        }
        info!("New language pool created");
        LanguagePool { set }
    }

    pub async fn lang_supported(&self, lang: &str) -> bool {
        self.set.contains(lang)
    }

    pub async fn get_supported(&self) -> Vec<String> {
        self.set.iter().map(|s| s.clone()).collect()
    }

    #[instrument]
    pub async fn new_snippet(&self, lang: String, code: String) -> Result<Snippet> {
        if self.lang_supported(&lang).await {
            let executor = format!("{}_executor", lang);
            info!("Language is supported");
            return Ok(Snippet::new(executor, code).await);
        }
        warn!("Language not supported: {}", lang);
        Err(anyhow!("Language is not yet supported"))
    }
}

pub struct Response {
    pub output: String,
    pub execution_time: Duration,
}

impl Response {
    pub async fn new(output_raw: Vec<u8>, execution_time: Duration) -> Self {
        let output = String::from_utf8(output_raw).unwrap();
        Self {
            output,
            execution_time,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Snippet {
    executor: String,
    code: String,
}

impl Snippet {
    pub async fn new(executor: String, code: String) -> Self {
        Self { executor, code }
    }
    #[instrument]
    pub async fn run(self) -> Result<Response> {
        info!("Running snippet");
        let start_time = Instant::now();

        let mut run = Command::new("docker")
            .args(&["run", "-i", "--rm"])
            .arg(&self.executor)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut input = run.stdin.take().unwrap();
        tokio::spawn(async move {
            input
                .write_all(&self.code.as_bytes())
                .await
                .map_err(|_| anyhow!("Could not pipe code to docker"))
        });
        let output = run.wait_with_output().await.unwrap();
        let execution_time = start_time.elapsed();
        let ms = execution_time.as_millis();
        info!("Snipped finished running in {}ms", ms);
        let output = if output.stderr.is_empty() {
            output.stdout
        } else {
            output.stderr
        };

        Ok(Response::new(output, execution_time).await)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::create_docker_executors;

    use super::*;

    #[tokio::test]
    async fn lang_pool_new() {
        let pool = LanguagePool::new().await;
        println!("{:?}", pool);
    }

    #[tokio::test]
    async fn lang_pool_supported() {
        let pool = LanguagePool::new().await;
        let rs_supported = pool.lang_supported("rust").await;
        assert!(rs_supported);

        let cpp_supported = pool.lang_supported("cpp").await;
        assert!(cpp_supported);

        let ktl_supported = pool.lang_supported("kotlin").await;
        assert!(!ktl_supported);
    }

    #[tokio::test]
    async fn lang_pool_new_snippet() {
        let pool = LanguagePool::new().await;
        let snippet = pool.new_snippet("cpp".into(), "test".into()).await.unwrap();
        let reference = Snippet::new("cpp_executor".into(), "test".into()).await;
        assert_eq!(snippet, reference);

        let snippet = pool
            .new_snippet("rust".into(), "test".into())
            .await
            .unwrap();
        let reference = Snippet::new("rust_executor".into(), "test".into()).await;
        assert_eq!(snippet, reference);
    }

    #[tokio::test]
    async fn snippet_run() {
        create_docker_executors().await;
        let snippet = Snippet::new("python_executor".into(), "print('python_test')".into()).await;
        let res = snippet.run().await.unwrap();
        println!("Res: {}", res.output);
        println!("Time: {}ms", res.execution_time.as_millis());
        assert_eq!("python_test\n", res.output);

        let snippet = Snippet::new("cpp_executor".into(), "#include<iostream>\nusing namespace std;\nint main() {cout <<\"test.cpp\" << endl; return 1;}".into()).await;
        let res = snippet.run().await.unwrap();
        println!("Res: {}", res.output);
        println!("Time: {}ms", res.execution_time.as_millis());
        assert_eq!("test.cpp\n", res.output);
    }
}
