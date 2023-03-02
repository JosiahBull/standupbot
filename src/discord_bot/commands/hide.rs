use serenity::{
    all::CommandInteraction,
    async_trait,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    prelude::Context,
};

use crate::state::AppState;

use super::{command::Command, util::CommandResponse};

pub struct HideCommand;

impl<'a> TryFrom<&'a CommandInteraction> for HideCommand {
    type Error = String;
    fn try_from(_: &'a CommandInteraction) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

#[async_trait]
impl<'a> Command<'a> for HideCommand {
    fn name() -> &'static str {
        "hide"
    }

    fn description() -> &'static str {
        "Creates a large message to hide previous messages in the chat"
    }

    fn get_application_command_options(i: CreateCommand) -> CreateCommand {
        i
    }

    #[allow(clippy::invisible_characters)]
    async fn handle_application_command<'b>(
        self,
        _: &'b CommandInteraction,
        _: &'b AppState,
        _: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        Ok(CommandResponse::ComplexSuccess(
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​​\n​​\n​​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​​\n​​\n​​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n​\n"))
        ))
    }
}
