use anyhow::{anyhow, Result};
use std::{collections::HashSet, process::Stdio};
use tokio::{fs, io::AsyncWriteExt, process::Command};

use crate::helper::{CMD_RGX, LANGS_PATH, LANG_POOL};

pub async fn run_pipeline(msg: &str) -> Result<String> {
    let (lang_str, code) = parse(msg).await?;
    let snippet = LANG_POOL.get().unwrap().new_snippet(lang_str, code).await?;
    let res = snippet.run().await?;
    Ok(res)
}

pub async fn parse(msg: &str) -> Result<(String, String)> {
    if !CMD_RGX.is_match(&msg) {
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
        LanguagePool { set }
    }
    // FIXME: fix this c++ bullshit
    pub async fn lang_supported(&self, lang: &str) -> bool {
        let lang = if lang == "c++" { "cpp" } else { lang };
        self.set.contains(lang)
    }

    pub async fn new_snippet(&self, lang: String, code: String) -> Result<Snippet> {
        if self.lang_supported(&lang).await {
            let lang = if lang == "c++" { "cpp".into() } else { lang };
            let executor = format!("{}_executor", lang);
            return Ok(Snippet::new(executor, code).await);
        }
        // TODO: log missing language
        Err(anyhow!("Language is not yet supported"))
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

    pub async fn run(self) -> Result<String> {
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
        let output = if output.stderr.is_empty() {
            output.stdout
        } else {
            output.stderr
        };

        Ok(String::from_utf8(output)?)
    }
}

#[cfg(test)]
mod tests {
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

        let cpp_supported = pool.lang_supported("c++").await;
        assert!(cpp_supported);

        let ktl_supported = pool.lang_supported("kotlin").await;
        assert!(!ktl_supported);
    }

    #[tokio::test]
    async fn lang_pool_new_snippet() {
        let pool = LanguagePool::new().await;
        let snippet = pool.new_snippet("c++".into(), "test".into()).await.unwrap();
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
        let snippet = Snippet::new("python_executor".into(), "print('test')".into()).await;
        let res = snippet.run().await.unwrap();
        println!("Res: {}", res);

        let snippet = Snippet::new("cpp_executor".into(), "#include<iostream>\nusing namespace std;\nint main() {cout <<\"test.cpp\" << endl; return 1;}".into()).await;
        let res = snippet.run().await.unwrap();
        println!("Res: {}", res);
    }
}
