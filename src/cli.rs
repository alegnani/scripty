use crate::helper::{CMD_RGX, LANG_POOL};

pub async fn parse_command(cmd: &str) -> Option<String> {
    if let Some(cmd) = cmd.strip_prefix("~") {
        let cmd = cmd.trim();
        let response = if CMD_RGX.is_match(&cmd) {
            run_command(cmd).await
        } else if cmd == "langs" {
            langs_command().await
        } else if cmd == "help" {
            help_command().await
        } else {
            err_command(cmd).await
        };
        return Some(response);
    }
    None
}

async fn run_command(msg: &str) -> String {
    let captures = CMD_RGX.captures(msg).unwrap();
    let lang = captures.get(1).unwrap().as_str().into();
    let code = captures.get(2).unwrap().as_str().into();
    let response = LANG_POOL
        .get()
        .unwrap()
        .from_code_file(lang, code)
        .await
        .unwrap()
        .run()
        .await
        .unwrap();
    format!(
        "```{}```\nIt took: {}ms",
        response.output,
        response.execution_time.as_millis()
    )
}

async fn help_command() -> String {
    HELP_MESSAGE.into()
}

async fn langs_command() -> String {
    let languages = LANG_POOL.get().unwrap().get_supported().await;
    let language_list = languages.join("\n - ");
    format!("Supported languages:\n - {}", language_list)
}

async fn err_command(cmd: &str) -> String {
    format!("Unrecognized command: {}\n{}", cmd, HELP_MESSAGE)
}

const HELP_MESSAGE: &str = "
scripty 0.1.0
A bot to execute code directly in a Discord channel

USAGE:
    ~help : Get this help page
    ~langs : Show a list of all supported languages
    ~run <CODE>: Runs the code snippet in the given language";
