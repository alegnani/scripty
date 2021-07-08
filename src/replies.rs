use serenity::{client::Context, model::channel::Message, utils::MessageBuilder};

use crate::languages::Response;

pub async fn run_timed_out(ctx: &Context, msg: &Message) {
    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("⚠️ Command timed out").description(
                    "Please keep in mind that code in this server times out after 5 seconds.",
                )
            })
        })
        .await;
}

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

pub async fn run_output(ctx: &Context, msg: &Message, response: Response) {
    if let Response::Output(res, exec_time) = response {
        let res = &format!("```{}```", res);
        let exec_time = &format!("{}ms", exec_time.as_millis());
        let response_to_long = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.field("Output", res, false)
                        .field("Execution time:", exec_time, true)
                })
            })
            .await
            .is_err();
        if response_to_long {
            let reply = MessageBuilder::new()
                .push_bold_line("Output")
                .push(res)
                .push("\n")
                .push_bold_line("Execution time:")
                .push(exec_time)
                .build();
            let _ = msg.channel_id.say(&ctx.http, reply).await;
        }
    }
}

pub async fn langs(ctx: &Context, msg: &Message, languages: Vec<String>) {
    let language_list = format!(" • {}", languages.join("\n • "));
    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.title("Supported languages").description(language_list))
        })
        .await;
}
