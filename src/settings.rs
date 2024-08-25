//! This module declares the structures used to serialize and deserialize the Discord server config.
//! The config allows for setting up server name, categories, and channels. Some fields are optional
//! to provide flexibility in the configuration.

use serde::Deserialize;
use std::collections::HashMap;

/// The main configuration structure for the Discord server.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Configuration related to the server.
    pub server: ServerConfig,
}

/// Configuration for the server, including its name and categories.
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// The name of the server.
    pub name: String,
    /// A map of category names to their respective configurations.
    pub categories: HashMap<String, CategoryConfig>,
    /// An optional description of the server.
    pub description: Option<String>,
    /// An optional icon URL for the server.
    pub icon_url: Option<String>,
}

/// Configuration for a category, including its channels.
#[derive(Debug, Deserialize)]
pub struct CategoryConfig {
    /// A map of channel names to their respective configurations.
    pub channels: HashMap<String, ChannelConfig>,
    /// An optional description of the category.
    pub description: Option<String>,
    /// Whether the category is marked as NSFW.
    pub nsfw: Option<bool>,
}

/// Configuration for an individual channel.
#[derive(Debug, Deserialize)]
pub struct ChannelConfig {
    /// The name of the channel.
    pub name: String,
    /// An optional topic for the channel.
    pub topic: Option<String>,
    /// Whether the channel is marked as NSFW.
    pub nsfw: Option<bool>,
    /// The position of the channel within the category.
    pub position: Option<u32>,
    /// Optional ID of the parent category if this is a sub-channel.
    pub parent_category: Option<String>,
}
