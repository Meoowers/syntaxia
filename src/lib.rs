//! Discord bot for configuration as code.

use error::CommandError;
use serenity::all::{Context, EventHandler, GuildId, Message, Ready};
use serenity::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod actions;
pub mod commands;
pub mod error;
pub mod settings;

/// The command handler that contains all data needed to run a command outside of the message context.
#[derive(Default, Clone)]
pub struct Handler {
    pub cooldown: Arc<RwLock<HashMap<GuildId, usize>>>,
    pub prefix: &'static str,
}

/// Parses the message content and executes the corresponding command if found.
pub async fn parse_commands(
    handler: &Handler,
    context: Context,
    message: Message,
) -> Result<(), error::Error> {
    if let Some(content) = message.content.clone().strip_prefix(handler.prefix) {
        if let Some((command_name, content)) = content.split_once(check_whitespace) {
            if command_name == "set" {
                let res = commands::set(handler, &context, &message, content.to_owned()).await;

                return execute_command(res, message, context).await;
            }
        }
    }
    Ok(())
}

/// Executes the given command and handles any errors that occur.
async fn execute_command<T>(
    err: Result<T, CommandError>,
    message: Message,
    context: Context,
) -> Result<(), error::Error> {
    if let Err(err) = err {
        match err {
            CommandError::User(error_message) => {
                message.channel_id.say(&context.http, error_message).await?;
            }
            CommandError::System(system_error) => {
                return Err(system_error);
            }
        }
    }
    Ok(())
}

/// Checks for whitespace or newline characters.
fn check_whitespace(x: char) -> bool {
    char::is_whitespace(x) || x == '\n'
}

#[async_trait]
impl EventHandler for Handler {
    /// Handles incoming messages and attempts to parse them as commands.
    async fn message(&self, ctx: Context, msg: Message) {
        if let Err(err) = parse_commands(self, ctx, msg).await {
            eprintln!("Error processing command: {:?}", err);
        }
    }

    /// Called when the bot is ready and connected to Discord.
    async fn ready(&self, _ctx: Context, _: Ready) {
        println!("The bot is ready");
    }
}
