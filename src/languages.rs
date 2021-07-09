use anyhow::{anyhow, Result};
use std::{collections::HashSet, process::Stdio};
use tokio::{
    fs,
    io::AsyncWriteExt,
    process::Command,
    time::{Duration, Instant},
};
use tracing::{error, info, instrument, warn};

use crate::utils::{CMD_RGX, LANGS_PATH};

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
        self.set.iter().cloned().collect()
    }

    #[instrument]
    pub async fn from_code_file(&self, lang: String, code: String) -> Result<Executable> {
        if self.lang_supported(&lang).await {
            let executor = format!("{}_executor", lang);
            info!("Language is supported");
            return Ok(Executable::new(executor, code).await);
        }
        warn!("Language not supported: {}", lang);
        Err(anyhow!("Language is not yet supported"))
    }
}

pub enum ContainerResponse {
    Output(String, Duration),
    Timeout,
}

impl ContainerResponse {
    pub async fn output(output_raw: Vec<u8>, execution_time: Duration) -> Self {
        Self::Output(String::from_utf8(output_raw).unwrap(), execution_time)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, Self::Output(_, _))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Executable {
    executor: String,
    code: String,
}

impl Executable {
    pub async fn new(executor: String, code: String) -> Self {
        Self { executor, code }
    }
    #[instrument]
    pub async fn run(self) -> Result<ContainerResponse> {
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
        let output = tokio::time::timeout(Duration::from_secs(5), run.wait_with_output()).await;
        match output {
            Ok(res) => {
                let res = res.unwrap();
                let execution_time = start_time.elapsed();
                let ms = execution_time.as_millis();
                info!("Snipped finished running in {}ms", ms);
                let res = if res.stderr.is_empty() {
                    res.stdout
                } else {
                    res.stderr
                };
                Ok(ContainerResponse::output(res, execution_time).await)
            }
            Err(_) => Ok(ContainerResponse::Timeout),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::docker_executors::create_docker_executors;

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
    async fn lang_pool_from_code_file() {
        let pool = LanguagePool::new().await;
        let snippet = pool
            .from_code_file("cpp".into(), "test".into())
            .await
            .unwrap();
        let reference = Executable::new("cpp_executor".into(), "test".into()).await;
        assert_eq!(snippet, reference);

        let snippet = pool
            .from_code_file("rust".into(), "test".into())
            .await
            .unwrap();
        let reference = Executable::new("rust_executor".into(), "test".into()).await;
        assert_eq!(snippet, reference);
    }

    #[tokio::test]
    async fn code_file_run() {
        create_docker_executors().await.unwrap();
        let snippet =
            Executable::new("python_executor".into(), "print('python_test')".into()).await;
        if let ContainerResponse::Output(ret, exec_time) = snippet.run().await.unwrap() {
            println!("Res: {}", &ret);
            println!("Time: {}ms", exec_time.as_millis());
            assert_eq!("python_test\n", ret);
        } else {
            panic!("Executor timed out");
        }

        let snippet = Executable::new("cpp_executor".into(), "#include<iostream>\nusing namespace std;\nint main() {cout <<\"cpp_test\" << endl; return 1;}".into()).await;
        if let ContainerResponse::Output(ret, exec_time) = snippet.run().await.unwrap() {
            println!("Res: {}", &ret);
            println!("Time: {}ms", exec_time.as_millis());
            assert_eq!("cpp_test\n", ret);
        } else {
            panic!("Executor timed out");
        }
    }
}
