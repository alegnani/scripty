use super::*;

pub async fn logic(ctx: &Context, msg: &Message) -> CommandResult {
    let _typing = msg.channel_id.start_typing(&ctx.http);
    let languages = LANG_POOL.get().unwrap().get_supported().await;
    reply(ctx, msg, languages).await;

    Ok(())
}

pub async fn reply(ctx: &Context, msg: &Message, languages: Vec<String>) {
    let language_list = format!(" • {}", languages.join("\n • "));
    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.title("Supported languages").description(language_list))
        })
        .await;
}
