use log::error;
use serenity::{
    all::{CommandInteraction, ComponentInteraction},
    async_trait,
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateInputText, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateModal,
    },
    prelude::Context,
};

use crate::state::AppState;

use super::{
    command::{Command, InteractionCommand},
    util::CommandResponse,
};

pub struct StandupCommand;

impl<'a> TryFrom<&'a CommandInteraction> for StandupCommand {
    type Error = String;
    fn try_from(_: &'a CommandInteraction) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

#[async_trait]
impl<'a> Command<'a> for StandupCommand {
    fn name() -> &'static str {
        "standup"
    }

    fn description() -> &'static str {
        "Run the weekly standups early, they will also be triggered Wednesday's at 3pm"
    }

    fn get_application_command_options(i: CreateCommand) -> CreateCommand {
        i
    }

    async fn handle_application_command<'b>(
        self,
        interaction: &'b CommandInteraction,
        state: &'b AppState,
        ctx: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        error!("interaction (app cmd): {:?}", interaction);
        // create a simple message response with a clickable button

        Ok(CommandResponse::ComplexSuccess(
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Click the button to start the standup")
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                        "standup-start",
                    )
                    .style(serenity::all::ButtonStyle::Primary)
                    .label("Start Standup")])]),
            ),
        ))
    }
}

#[async_trait]
impl InteractionCommand<'_> for StandupCommand {
    async fn answerable<'b>(
        interaction: &'b ComponentInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> bool {
        return true; //TODO
    }

    async fn interaction<'b>(
        interaction: &'b ComponentInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        // Ok(CommandResponse::ComplexSuccess(

        // ))

        interaction
            .create_response(
                context,
                CreateInteractionResponse::Modal(
                    CreateModal::new("hellothere", "standups").components({
                        vec![
                            CreateActionRow::InputText(CreateInputText::new(
                                serenity::all::InputTextStyle::Short,
                                "What did you do this week?",
                                "standup-weekly-task",
                            )),
                            CreateActionRow::InputText(CreateInputText::new(
                                serenity::all::InputTextStyle::Short,
                                "What are you going to do next week?",
                                "standup-next-week-task",
                            )),
                        ]
                    }),
                ),
            )
            .await
            .unwrap();

        Ok(CommandResponse::NoResponse)
    }
}
