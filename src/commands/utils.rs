use super::*;

pub async fn wrong_syntax(ctx: &Context, msg: &Message, command_name: &str) {
    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("⚠️ Wrong syntax for command {}", command_name))
                    .description(format!(
                        "Please refer to the help page of this command with: `~help {}`",
                        command_name
                    ))
            })
        })
        .await;
}

pub async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("⚠️ Unknown command {}", unknown_command_name))
                    .description("Please refer to the help page with: `~help`")
            })
        })
        .await;
}
