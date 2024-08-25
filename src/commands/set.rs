use crate::settings::Config;
use crate::{actions, CommandError, Context, Handler};
use regex::Regex;
use serde_yaml;
use serenity::all::Message;

pub async fn set(
    _: &Handler,
    context: &Context,
    message: &Message,
    content: String,
) -> Result<(), CommandError> {
    let guild_id = message
        .guild_id
        .ok_or_else(|| CommandError::User("Cannot run this outside of a Guild.".into()))?;

    let re = Regex::new(r"```yaml\n([\s\S]*?)\n```").unwrap();

    let yaml_content = re
        .captures(&content)
        .and_then(|capture| capture.get(1))
        .map(|x| x.as_str())
        .unwrap_or_else(|| &content);

    let config: Config = serde_yaml::from_str(yaml_content).map_err(|_| {
        CommandError::User("Invalid YAML structure for configuring the server.".to_string())
    })?;

    message
        .channel_id
        .say(&context.http, "Configuring...".to_string())
        .await?;

    match actions::config::run(config, context.clone(), guild_id).await {
        Ok(_) => {
            message
                .channel_id
                .say(&context.http, "Finished...".to_string())
                .await?;
        }
        Err(err) => {
            eprint!("Failed at configuring");
            message
                .channel_id
                .say(
                    &context.http,
                    format!("Could not complete the setup. {:}", err),
                )
                .await?;
        }
    };

    Ok(())
}
