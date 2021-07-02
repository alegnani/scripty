use crate::helper::{CMD_RGX, LANG_POOL, SCRIPTS_PATH};
use crate::language::Language;
use anyhow::{anyhow, Result};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use uuid::Uuid;

pub async fn pipeline(cmd: &str) -> Result<String> {
    let r_task = RawTask::from_command(cmd).await?;
    let task = r_task.to_task().await?;
    let env = Env::from(task).await?;
    let reply = env.run().await?;
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
        env.create_launch().await?;
        Ok(env)
    }

    async fn create_code(&self, code: String) -> Result<()> {
        let file_path = format!("{}/main.{}", &self.path, self.lang.get_extension().await);
        let mut f = fs::File::create(file_path)
            .await
            .map_err(|_| anyhow!("Could not create code file"))?;
        f.write_all(&code.as_bytes())
            .await
            .map_err(|_| anyhow!("Could not write to code file"))?;
        Ok(())
    }

    async fn create_launch(&self) -> Result<()> {
        let file_path = format!("{}/run.sh", &self.path);
        let mut f = fs::File::create(&file_path)
            .await
            .map_err(|_| anyhow!("Could not create launch file"))?;
        f.write_all(self.lang.get_launch().await.as_bytes())
            .await
            .map_err(|_| anyhow!("Could not write to launch file"))?;
        if !Command::new("chmod")
            .arg("+x")
            .arg(file_path)
            .spawn()?
            .wait()
            .await?
            .success()
        {
            return Err(anyhow!("Could not add exec permission to launch file"));
        }
        Ok(())
    }

    pub async fn run(self) -> Result<String> {
        let output =Command::new("/bin/sh").arg("run.sh").current_dir(&self.path).output().await.unwrap();
        let output = if output.stderr.is_empty() {
            output.stdout
        } else {
            output.stderr
        };
        self.clean_up().await?;
        Ok(String::from_utf8(output).unwrap())
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
        let t = r.to_task().await.unwrap();

        let r = RawTask::from_command(incorrect).await.unwrap();
        let t = r.to_task().await.unwrap_err();
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
        let output = env.run().await.unwrap();
        assert_eq!(output, "hi\n");
    }
}
