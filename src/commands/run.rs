use serenity::utils::MessageBuilder;

use crate::languages::Response;

use super::*;

pub async fn logic(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let _typing = msg.channel_id.start_typing(&ctx.http);
    let code_string = args.remains().unwrap_or("");
    if CMD_RGX.is_match(code_string) {
        let captures = CMD_RGX.captures(&msg.content).unwrap();
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

        if response.is_output() {
            reply_output(ctx, msg, response).await;
        } else {
            reply_timed_out(ctx, msg).await;
        };
    } else {
        utils::wrong_syntax(ctx, msg, "run").await;
    }
    Ok(())
}

async fn reply_timed_out(ctx: &Context, msg: &Message) {
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

async fn reply_output(ctx: &Context, msg: &Message, response: Response) {
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
