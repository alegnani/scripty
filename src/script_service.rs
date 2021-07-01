use anyhow::{anyhow, Result};
use regex::Regex;

#[derive(Debug)]
struct RawScript {
    language: String,
    code: String,
}

async fn parse_command(cmd: &str) -> Result<RawScript> {
    let rgx = Regex::new(r"^~run *```([a-z]*)\n((?s).*)\n```").unwrap();
    if !rgx.is_match(&cmd) {
        return Err(anyhow!("RegEx is not match"));
    }
    println!("RegEx is a match!");
    let captures = rgx.captures(cmd).unwrap();
    let language = captures.get(1).unwrap().as_str().into();
    let code = captures.get(2).unwrap().as_str().into();
    Ok(RawScript { language, code })
}

async fn create_exe(raw: RawScript) {}

async fn execute(script: String) {}

// take code and language and create file
// return closure for execution

//execute closure and return values

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_command_correct() {
        let correct = "~run ```python\ntest\n```";
        let raw = parse_command(correct).await.unwrap();
        assert_eq!(raw.language, "python");
        assert_eq!(raw.code, "test");
    }

    #[tokio::test]
    async fn parse_command_incorrect() {
        let incorrect = "~run ```\ntest\n";
        let err = parse_command(incorrect).await.unwrap_err();
        assert_eq!(err.to_string(), "RegEx is not match");
    }
}
