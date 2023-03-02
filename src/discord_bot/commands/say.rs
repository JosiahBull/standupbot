use serenity::{
    all::{CommandInteraction, CommandOptionType},
    async_trait,
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateMessage,
    },
    prelude::Context,
};

use crate::state::AppState;

use super::{
    command::Command,
    util::{CommandResponse, FailureMessageKind},
};

pub struct SayCommand<'a> {
    message: &'a str,
}

impl<'a> TryFrom<&'a CommandInteraction> for SayCommand<'a> {
    type Error = String;
    fn try_from(interaction: &'a CommandInteraction) -> Result<Self, Self::Error> {
        let message = interaction
            .data
            .options
            .get(0)
            .ok_or("No message provided")?
            .value
            .as_str()
            .ok_or("No message provided")?;
        Ok(Self { message })
    }
}

#[async_trait]
impl<'a> Command<'a> for SayCommand<'a> {
    fn name() -> &'static str {
        "say"
    }

    fn description() -> &'static str {
        "Says whatever you want!"
    }

    fn get_application_command_options(i: CreateCommand) -> CreateCommand {
        i.add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "text",
                "What you want the bot to say",
            )
            .required(true)
            .max_length(1900)
            .to_owned(),
        )
    }

    async fn handle_application_command<'b>(
        self,
        interaction: &'b CommandInteraction,
        _: &'b AppState,
        ctx: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        if let Err(e) = interaction
            .channel_id
            .send_message(ctx, CreateMessage::new().content(self.message))
            .await
        {
            return Err(CommandResponse::ComplexFailure {
                response: String::from("Failed to use /say due to error"),
                kind: FailureMessageKind::Error,
                log_message: e.to_string(),
            });
        }

        Ok(CommandResponse::ComplexSuccess(
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("I will send: {}", self.message))
                    .ephemeral(true),
            ),
        ))
    }
}
