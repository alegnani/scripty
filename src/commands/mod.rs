pub mod langs;
pub mod log;
pub mod run;
pub mod utils;

use crate::utils::{CMD_RGX, LANG_POOL};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
