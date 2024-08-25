use crate::error::Error as SystemError;
use crate::settings::{CategoryConfig, ChannelConfig, Config, ServerConfig};
use serenity::all::*;
use std::collections::HashMap;
use thiserror::Error;

/// Represents all possible errors that can occur during the configuration process.
#[derive(Debug, Error)]
pub enum Error {
    /// Occurs when the guild is not found in the cache or context.
    #[error("PartialGuild not found")]
    GuildNotFound,

    /// Occurs when a Serenity API call fails.
    #[error("Serenity API error: {0}")]
    SerenityError(#[from] serenity::Error),

    /// Occurs when a category or channel is not found.
    #[error("Category or channel not found: {0}")]
    NotFound(String),

    /// Occurs when a category or channel creation fails.
    #[error("Failed to create category or channel: {0}")]
    CreationFailed(String),

    /// Occurs when updating a category or channel fails.
    #[error("Failed to update category or channel: {0}")]
    UpdateFailed(String),

    /// Occurs when invalid data is provided.
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Creates or updates categories and channels in the guild according to the provided configuration.
pub async fn run(config: Config, context: Context, guild_id: GuildId) -> Result<(), SystemError> {
    let mut guild = context
        .http
        .get_guild(guild_id)
        .await
        .map_err(|_| Error::GuildNotFound)?
        .to_owned();

    let mut channels = guild.channels(&context.http).await?;

    update_guild_info(&config.server, &context, &mut guild).await?;
    process_categories(
        &config.server.categories,
        &context,
        &mut channels,
        &mut guild,
    )
    .await?;

    Ok(())
}

/// Updates the guild's name and icon if they differ from the configuration.
async fn update_guild_info(
    server_config: &ServerConfig,
    context: &Context,
    guild: &mut PartialGuild,
) -> Result<(), SystemError> {
    if guild.name != server_config.name
        || guild.icon_url().as_ref() != server_config.icon_url.as_ref()
    {
        let mut edit = EditGuild::default();
        edit = edit.name(&server_config.name);

        guild.edit(&context.http, edit).await?;
    }
    Ok(())
}

/// Processes categories by creating or updating them and then processes their channels.
async fn process_categories(
    categories: &HashMap<String, CategoryConfig>,
    context: &Context,
    channels: &mut HashMap<ChannelId, GuildChannel>,
    guild: &mut PartialGuild,
) -> Result<(), SystemError> {
    for (category_name, category_config) in categories {
        let category_id =
            find_or_create_category(category_name, category_config, context, channels, guild)
                .await?;
        process_channels(
            context,
            &category_config.channels,
            channels,
            guild,
            &category_id,
        )
        .await?;
    }
    Ok(())
}

/// Finds an existing category or creates a new one if it does not exist.
async fn find_or_create_category(
    category_name: &str,
    category_config: &CategoryConfig,
    context: &Context,

    channels: &mut HashMap<ChannelId, GuildChannel>,
    guild: &PartialGuild,
) -> Result<ChannelId, SystemError> {
    if let Some(category_channel) = channels
        .values()
        .find(|c| c.kind == ChannelType::Category && c.name == category_name)
    {
        update_category_if_needed(&category_channel.id, category_config, context).await?;
        Ok(category_channel.id)
    } else {
        let mut edit = CreateChannel::new(category_name);
        edit = edit.kind(ChannelType::Category);

        if let Some(description) = &category_config.description {
            edit = edit.topic(description);
        }

        if let Some(nsfw) = category_config.nsfw {
            edit = edit.nsfw(nsfw);
        }

        let new_category = guild.create_channel(&context.http, edit).await?;
        Ok(new_category.id)
    }
}

/// Updates the category if there are any changes in its configuration.
async fn update_category_if_needed(
    category_id: &ChannelId,
    category_config: &CategoryConfig,
    context: &Context,
) -> Result<(), Error> {
    if category_config.description.is_some() || category_config.nsfw.is_some() {
        let mut edit = EditChannel::default();

        if let Some(description) = &category_config.description {
            edit = edit.topic(description);
        }

        if let Some(nsfw) = &category_config.nsfw {
            edit = edit.nsfw(*nsfw);
        }

        category_id.edit(&context.http, edit).await?;
    }
    Ok(())
}

/// Processes channels within a category by creating or updating them.
async fn process_channels(
    context: &Context,
    config: &HashMap<String, ChannelConfig>,
    channels: &mut HashMap<ChannelId, GuildChannel>,
    guild: &mut PartialGuild,
    category_id: &ChannelId,
) -> Result<(), SystemError> {
    for (channel_name, channel_config) in config {
        find_or_create_channel(
            channel_name,
            channel_config,
            context,
            channels,
            guild,
            category_id,
        )
        .await?;
    }
    Ok(())
}

/// Finds an existing channel or creates a new one if it does not exist.
async fn find_or_create_channel(
    channel_name: &str,
    channel_config: &ChannelConfig,
    context: &Context,
    channels: &mut HashMap<ChannelId, GuildChannel>,
    guild: &mut PartialGuild,
    category_id: &ChannelId,
) -> Result<(), SystemError> {
    if let Some(channel) = channels.values().find(|c| {
        c.kind == ChannelType::Text && c.name == channel_name && c.parent_id == Some(*category_id)
    }) {
        update_channel_if_needed(&channel.id, channel_config, context).await?;
    } else {
        create_channel(channel_name, channel_config, context, guild, category_id).await?;
    }
    Ok(())
}

/// Updates the channel if there are any changes in its configuration.
async fn update_channel_if_needed(
    channel_id: &ChannelId,
    channel_config: &ChannelConfig,
    context: &Context,
) -> Result<(), SystemError> {
    let mut edit = EditChannel::new();

    edit = edit.name(channel_config.name.clone());

    if let Some(topic) = &channel_config.topic {
        edit = edit.topic(topic);
    }

    if let Some(nsfw) = channel_config.nsfw {
        edit = edit.nsfw(nsfw);
    }

    if let Some(position) = channel_config.position {
        edit = edit.position(position as u16);
    }

    channel_id.edit(&context.http, edit).await?;
    Ok(())
}

/// Creates a new channel with the specified configuration.
async fn create_channel(
    channel_name: &str,
    channel_config: &ChannelConfig,
    context: &Context,
    guild: &mut PartialGuild,
    category_id: &ChannelId,
) -> Result<ChannelId, SystemError> {
    let mut edit = CreateChannel::new(channel_name);

    edit = edit.kind(ChannelType::Text);
    edit = edit.category(*category_id);

    if let Some(topic) = &channel_config.topic {
        edit = edit.topic(topic);
    }
    if let Some(nsfw) = channel_config.nsfw {
        edit = edit.nsfw(nsfw);
    }
    if let Some(position) = channel_config.position {
        edit = edit.position(position as u16);
    }

    let new_channel = guild.create_channel(&context.http, edit).await?;
    Ok(new_channel.id)
}
