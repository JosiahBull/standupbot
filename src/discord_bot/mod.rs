//! This module is used for managing everything related to the actual discord server, and the bot itself.
//! The bot is built on top of the Serenity discord crate.

mod commands;
mod guilds;
mod handler;
mod manager;
mod utils;

pub use manager::{DiscordBot, DiscordBotBuilder};
