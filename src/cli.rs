use std::collections::HashSet;

use crate::helper::{CMD_RGX, LANG_POOL};
use crate::replies;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::macros::{command, group, help, hook};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::prelude::Activity;
use tracing::{info, warn};

pub async fn get_client(token: &str) -> Client {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .help(&HELP)
        .group(&EVERYONE_GROUP);
    //.group(&ADMINS_GROUP); // TODO: add admin panel
    let client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .unwrap();
    client
}

#[group]
#[commands(langs, run)]
struct Everyone;

#[group]
#[owners_only]
#[commands(logs)]
struct Admins;

#[help]
#[individual_command_tip = "Hello World!\nThis is scripty, a bot allowing you to easily run your favourite code snippets directly in Discord!\nIf you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(1)]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    println!("Help invoked!");
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[command("logs")]
async fn logs(_ctx: &Context, _msg: &Message) -> CommandResult {
    // TODO
    Ok(())
}

#[command("langs")]
#[description = "Returns a list of the supported languages"]
#[usage = ""]
#[num_args(0)]
async fn langs(ctx: &Context, msg: &Message) -> CommandResult {
    let _typing = msg.channel_id.start_typing(&ctx.http);
    let languages = LANG_POOL.get().unwrap().get_supported().await;
    replies::langs(ctx, msg, languages).await;

    Ok(())
}

#[command("run")]
#[description = "Runs the given"]
#[example = " ```python\nprint('scripty is the best')\n```\n"]
#[usage = "<CODE_MARKDOWN>"]
async fn run(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
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
            replies::run_output(ctx, msg, response).await;
        } else {
            replies::run_timed_out(ctx, msg).await;
        };
    } else {
        replies::wrong_syntax(ctx, msg, "run").await;
    }
    Ok(())
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    replies::unknown_command(ctx, msg, unknown_command_name).await;
    warn!("Invalid command used: {}", unknown_command_name);
}

// #[hook]
// async fn normal_message(_ctx: &Context, msg: &Message) {
//     println!("Message is not a command '{}'", msg.content);
// }

// #[hook]
// async fn delay_action(ctx: &Context, msg: &Message) {
//     // You may want to handle a Discord rate limit if this fails.
//     let _ = msg.react(ctx, '⏱').await;
// }

// #[hook]
// async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
//     if let DispatchError::Ratelimited(info) = error {
//         // We notify them only once.
//         if info.is_first_try {
//             let _ = msg
//                 .channel_id
//                 .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
//                 .await;
//         }
//     }
// }

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data: serenity::model::prelude::Ready) {
        ctx.set_activity(Activity::playing("Counting 1s and 0s..."))
            .await;
        info!("Scripty is online and ready");
    }
}
