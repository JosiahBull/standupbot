use serenity::{
    all::CommandInteraction,
    async_trait,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    prelude::Context,
};

use crate::state::AppState;

use super::{command::Command, util::CommandResponse};

pub struct PingCommand;

impl<'a> TryFrom<&'a CommandInteraction> for PingCommand {
    type Error = String;
    fn try_from(_: &'a CommandInteraction) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

#[async_trait]
impl<'a> Command<'a> for PingCommand {
    fn name() -> &'static str {
        "ping"
    }

    fn description() -> &'static str {
        "Pings the bot, expect a pong response."
    }

    fn get_application_command_options(i: CreateCommand) -> CreateCommand {
        i
    }

    async fn handle_application_command<'b>(
        self,
        _: &'b CommandInteraction,
        _: &'b AppState,
        _: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        Ok(CommandResponse::ComplexSuccess(
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Pong!")
                    .ephemeral(true),
            ),
        ))
    }
}
