mod discord_bot;

mod healthcheck;

mod logging;
mod state;

use log::{error, info};
use std::process::exit;

use crate::{discord_bot::DiscordBot, logging::configure_logger, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure_logger()?;

    let discord_token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let state = AppState::new().await?;

    info!("spawning discord handler");
    let discord_state = state.clone();
    let discord_handle = tokio::task::spawn(async move {
        let builder = DiscordBot::builder()
            .discord_token(discord_token)
            .state(discord_state)
            .build();

        let bot = match builder {
            Ok(bot) => bot,
            Err(e) => {
                error!("failed to build discord bot: {}", e);
                return;
            }
        };

        if let Err(e) = bot.run().await {
            error!("failed to run discord bot: {}", e);
            exit(1);
        }

        info!("discord bot shut down");
    });

    info!("creating healthcheck server");
    let healthcheck_state = state.clone();
    let healthcheck_handle = tokio::task::spawn(async move {
        let builder = healthcheck::Healthcheck::builder()
            .state(healthcheck_state)
            .build();

        let mut server = match builder.await {
            Ok(server) => server,
            Err(e) => {
                error!("failed to build healthcheck server: {}", e);
                return;
            }
        };

        server.run().await;

        info!("healthcheck server shut down");
    });

    tokio::pin!(discord_handle);
    tokio::pin!(healthcheck_handle);

    loop {
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                info!("received ctrl-c, shutting down");
                break;
            }

            _ = &mut discord_handle => {
                info!("discord handler shut down");
                break;
            }

            _ = healthcheck_handle => {
                info!("healthcheck server shut down");
                break;
            }
        }
    }

    info!("global TomBot shutdown");

    Ok(())
}
