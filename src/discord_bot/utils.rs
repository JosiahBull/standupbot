use serenity::prelude::TypeMapKey;

/// represents the unique identifier that represents the user-id of this discord bot
pub struct BotDiscordId(u64);

impl BotDiscordId {
    /// create a new bot identifier
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// get the discord id of the bot
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl TypeMapKey for BotDiscordId {
    type Value = BotDiscordId;
}
