use std::collections::HashSet;

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

use crate::commands;

pub async fn get_client(token: &str) -> Client {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .before(before)
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

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data: serenity::model::prelude::Ready) {
        ctx.set_activity(Activity::playing("Counting 1s and 0s..."))
            .await;
        info!("Scripty is online and ready");
    }
}

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
async fn langs(ctx: &Context, msg: &Message) -> CommandResult {
    commands::langs::logic(ctx, msg).await
}

#[command("run")]
#[description = "Runs the given"]
#[example = " ```python\nprint('scripty is the best')\n```\n"]
#[usage = "<CODE_MARKDOWN>"]
async fn run(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    commands::run::logic(ctx, msg, args).await
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
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    commands::utils::unknown_command(ctx, msg, unknown_command_name).await;
    warn!("Invalid command used: {}", unknown_command_name);
}
