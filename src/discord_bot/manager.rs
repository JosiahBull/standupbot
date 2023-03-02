//! The global manager for the bot, which manages all guilds as individual tasks
//! and coordinates events between them.

use std::{collections::HashMap, ops::DerefMut, time::Duration};

use log::{error, warn};
use serenity::{
    all::Interaction,
    futures::{stream::FuturesUnordered, StreamExt},
    model::prelude::Message,
    prelude::{GatewayIntents, TypeMapKey},
    Client,
};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, UnboundedSender},
};

use super::{guilds::GuildHandler, handler::Handler};

/// An event that may occur between the various discord services
#[derive(Debug)]
pub enum DiscordEvent {
    /// a guild has been added, and must be managed
    NewGuild(GuildHandler),
    /// a guild was deleted and should no longer be managed
    DeletedGuild(u64),
    /// an interaction has been received from the user, and must be handled by a specific guild
    // enum variant boxed, as is quite large and so should be heap-allocated
    Interaction(Box<Interaction>),
    /// a new message received from any guild
    Message(Box<Message>),
    /// a shutdown command to be sent to a guild, when received the guild should cease all activity and shut down
    Shutdown,
}

/// A channel that can be used to send messages between guild handlers and the master discord process
#[derive(Clone)]
pub struct InternalSender(UnboundedSender<DiscordEvent>);

impl TypeMapKey for InternalSender {
    type Value = InternalSender;
}

impl std::ops::Deref for InternalSender {
    type Target = UnboundedSender<DiscordEvent>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InternalSender {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A builder for the discord bot
pub struct DiscordBotBuilder<T> {
    /// the discord token to use for authentication with the discord api
    discord_token: Option<String>,
    /// the database to use for storing data
    app_state: Option<T>,
}

impl<T> DiscordBotBuilder<T> {
    /// set the token for the discord bot
    pub fn discord_token(mut self, token: String) -> Self {
        self.discord_token = Some(token);
        self
    }

    /// The application state for the bot. This is required for the bot to
    /// communicate with a database.
    pub fn state(mut self, app_state: T) -> Self {
        self.app_state = Some(app_state);
        self
    }

    /// Build the bot, and create a [DiscordBot] instance.
    pub fn build(self) -> Result<DiscordBot<T>, String> {
        let discord_token = match self.discord_token {
            Some(token) => token,
            None => return Err("No token provided".to_string()),
        };

        let app_state = match self.app_state {
            Some(app_state) => app_state,
            None => return Err("No app state provided".to_string()),
        };

        Ok(DiscordBot {
            discord_token,
            app_state,
        })
    }
}

impl<T> Default for DiscordBotBuilder<T> {
    fn default() -> Self {
        DiscordBotBuilder {
            discord_token: None,
            app_state: None,
        }
    }
}

/// A discord bot global manager, runs asynchronously communicating via channels
/// supports graceful and force shutdown, among many other thingss
pub struct DiscordBot<T> {
    /// the discord token to use for authentication with the discord api
    discord_token: String,
    /// the database to use for storing data
    app_state: T,
}

impl<T: Send + Sync + 'static + Clone + TypeMapKey<Value = T>> DiscordBot<T> {
    /// Get a builder to setup a new discord bot
    pub fn builder() -> DiscordBotBuilder<T> {
        DiscordBotBuilder::default()
    }

    /// Start the discord bot, this will connect to discords api and create internal
    /// handlers as required.
    /// Will exit when the bot has fully disconnected from all services.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let intents = GatewayIntents::all();

        let mut client = Client::builder(&self.discord_token, intents)
            .event_handler(Handler)
            .await?;

        let (i_tx, mut i_rx) = unbounded_channel();

        // scoping this off means we'll drop the write guard properly
        {
            let mut data = client.data.write().await;
            data.insert::<InternalSender>(InternalSender(i_tx));
            // data.insert::<BotDiscordId>(BotDiscordId::new(client.user_id.0));
            data.insert::<T>(self.app_state.clone());
        }

        let handle = tokio::task::spawn(async move {
            let mut thread_handles = FuturesUnordered::new();
            let mut guild_handlers: HashMap<u64, GuildHandler> = HashMap::default();

            loop {
                select! {
                    Some(i_e) = i_rx.recv() => {
                        match i_e {
                            DiscordEvent::NewGuild(handler) => {
                                // finish creating the handler
                                let key: u64 = handler.guild_id.into();
                                if guild_handlers.contains_key(&key) {
                                    panic!("tried to double handle a guild");
                                    // if let Err(e) = handler.close(Duration::from_secs(0)).await {
                                    //     error!("failed to close a guild handler {}", e);
                                    // }
                                }
                                guild_handlers.insert(key, handler);
                            },
                            DiscordEvent::DeletedGuild(guild) => {
                                // remove guild handler
                                let mut g_h = match guild_handlers.remove(&guild) {
                                    Some(s) => s,
                                    None => {
                                        error!("tried to remove non-existant guild");
                                        return;
                                    }
                                };

                                let t_h = tokio::task::spawn(async move {
                                    if let Err(e) = g_h.close(Duration::from_secs(5)).await {
                                        error!("failed to close a guild handler {}", e);
                                    }
                                });

                                thread_handles.push(t_h);
                            },
                            DiscordEvent::Interaction(interaction) => {
                                let guild_id: Option<serenity::model::id::GuildId> = match *interaction {
                                    Interaction::Ping(_) => {
                                        error!("got ping application command, which was not handled");
                                        continue;
                                    },
                                    Interaction::Command(ref c) => c.guild_id,
                                    Interaction::Component(ref c) => c.guild_id,
                                    Interaction::Autocomplete(ref c) => c.guild_id,
                                    Interaction::Modal(ref c) => c.guild_id,
                                };

                                let guild_id: u64 = match guild_id {
                                    Some(g_id) => g_id.0.into(),
                                    None => {
                                        error!("got interaction without guild id");
                                        continue;
                                    }
                                };

                                let g_h = match guild_handlers.get(&guild_id) {
                                    Some(s) => s.internal_tx.clone(),
                                    None => {
                                        error!("tried to handle message for non-existant guild id {}", guild_id);
                                        return;
                                    }
                                };
                                if let Err(e) = g_h.send(DiscordEvent::Interaction (interaction)) {
                                    error!("failed to send interaction to guild handler {}", e);
                                }
                            },
                            DiscordEvent::Message(message) => {
                                let guild_id: u64 = match message.guild_id {
                                    Some(g_id) => g_id.into(),
                                    None => {
                                        warn!("got message without guild id");
                                        continue;
                                    }
                                };

                                let g_h = match guild_handlers.get(&guild_id) {
                                    Some(s) => s.internal_tx.clone(),
                                    None => {
                                        error!("tried to handle message for non-existant guild id {}", guild_id);
                                        return;
                                    }
                                };

                                if let Err(e) = g_h.send(DiscordEvent::Message(message)) {
                                    error!("failed to send message to guild handler {}", e);
                                }
                            }
                            e => error!("unexpected discord event received {:?}", e),
                        }
                    },
                    _ = thread_handles.next(), if !thread_handles.is_empty() => {} //drain the handles as they complete
                    else => {
                        panic!("both receivers closed without breaking the loop, this indicates a failure")
                    }
                }
            }
        });

        client.start().await?;

        // once the discord bot has shutdown, we can expect to close the manager,
        // so we need to clean up the task for it.
        handle.abort();

        Ok(())
    }
}
