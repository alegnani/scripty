use crate::helper::{CMD_RGX, LANG_POOL, SCRIPTS_PATH};
use crate::language::Language;
use anyhow::{anyhow, Result};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use std::convert::TryInto;
use std::process::Stdio;
use uuid::Uuid;

pub async fn pipeline(cmd: &str) -> Result<String> {
    let r_task = RawTask::from_command(cmd).await?;
    let task = r_task.to_task().await?;
    let env = Env::from(task).await?;
    let reply = env.run_docker().await?;
    Ok(reply)
}

#[derive(Debug)]
pub struct RawTask {
    code: String,
    lang_str: String,
}

impl RawTask {
    pub async fn from_command(cmd: &str) -> Result<Self> {
        if !CMD_RGX.is_match(&cmd) {
            return Err(anyhow!("Regex does not match command"));
        }
        let captures = CMD_RGX.captures(cmd).unwrap();
        let lang_str = captures.get(1).unwrap().as_str().into();
        let code = captures.get(2).unwrap().as_str().into();
        Ok(Self { code, lang_str })
    }

    pub async fn to_task(self) -> Result<Task<'static>> {
        let lang = LANG_POOL
            .get()
            .unwrap()
            .get(&self.lang_str)
            .await
            .map_err(|_| anyhow!("Language is not yet supported"))?;
        Ok(Task {
            code: self.code,
            lang,
        })
    }
}

#[derive(Debug)]
pub struct Task<'a> {
    code: String,
    lang: &'a Language,
}

#[derive(Debug)]
pub struct Env<'a> {
    uuid: Uuid,
    path: String,
    lang: &'a Language,
}

impl<'a> Env<'a> {
    pub async fn from(t: Task<'a>) -> Result<Env<'a>> {
        let uuid = Uuid::new_v4();
        let path = format!("{}/{}", *SCRIPTS_PATH, &uuid);
        if fs::create_dir(&path).await.is_err() 
        {
            return Err(anyhow!("Could not create directory for container"));
        }
        let env = Env {
            uuid,
            path,
            lang: t.lang,
        };
        env.create_code(t.code).await?;
        Ok(env)
    }

    async fn create_code(&self, code: String) -> Result<()> {
        let file_path = format!("{}/code", &self.path);
        let mut f = fs::File::create(file_path)
            .await
            .map_err(|_| anyhow!("Could not create code file"))?;
        f.write_all(&code.as_bytes())
            .await
            .map_err(|_| anyhow!("Could not write to code file"))?;
        Ok(())
    }

    pub async fn run_docker(self) -> Result<String> {
        let executor = format!("{}_executor", &self.lang.get_name().await);
        let mut read_code = Command::new("cat").arg("code").current_dir(&self.path).stdout(Stdio::piped()).spawn()?;
        let pipe: Stdio = read_code.stdout.take().unwrap().try_into()?;
        let docker_run = Command::new("docker").arg("run").arg("-i").arg("--rm").arg(executor).stdin(pipe).output().await?;
        let output = if docker_run.stderr.is_empty() {
            docker_run.stdout
        } else {
            docker_run.stderr
        };
        self.clean_up().await?;
        Ok(String::from_utf8(output)?)
    }

    pub async fn clean_up(self) -> Result<()> {
        fs::remove_dir_all(self.path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper::LANG_POOL;
    use crate::language_pool::LanguagePool;

    async fn setup() {
        let _ = LANG_POOL.set(LanguagePool::new().await);
    }
    #[tokio::test]
    async fn raw_task_from() {
        setup().await;
        let correct = "~run ```python\ntest\n```";
        let incorrect = "~run ```\ntest\n";

        let task = RawTask::from_command(correct).await.unwrap();
        assert_eq!(task.code, "test");
        assert_eq!(task.lang_str, "python");

        RawTask::from_command(incorrect).await.unwrap_err();
    }

    #[tokio::test]
    async fn to_task() {
        setup().await;
        let correct = "~run ```python\ntest\n```";
        let incorrect = "~run ```kotlin\ntest\n```";

        let r = RawTask::from_command(correct).await.unwrap();
        let _ = r.to_task().await.unwrap();

        let r = RawTask::from_command(incorrect).await.unwrap();
        let _ = r.to_task().await.unwrap_err();
    }

    #[tokio::test]
    async fn env_from() {
        setup().await;
        let correct = "~run ```python\ntest\n```";

        let r = RawTask::from_command(correct).await.unwrap();
        let t = r.to_task().await.unwrap();
        let env = Env::from(t).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        env.clean_up().await.unwrap();
    }

    #[tokio::test]
    async fn env_run() {
        setup().await;
        let cmd = "~run ```python\nprint('hi')\n```";
        let env = Env::from(
            RawTask::from_command(cmd)
                .await
                .unwrap()
                .to_task()
                .await
                .unwrap(),
        )
        .await
        .unwrap();
        println!("{}", env.path);
        let output = env.run_docker().await.unwrap();
        assert_eq!(output, "hi\n");
    }
}
