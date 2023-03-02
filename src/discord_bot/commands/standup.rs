use log::error;
use serenity::{
    all::{CommandInteraction, ComponentInteraction, ModalInteraction},
    async_trait,
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateInputText, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateMessage, CreateModal,
    },
    prelude::Context,
};

use crate::state::AppState;

use super::{
    command::{Command, InteractionCommand, ModalSubmit},
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
        _: &'b AppState,
        _: &'b Context,
    ) -> bool {
        interaction.data.custom_id == "standup-start"
    }

    async fn interaction<'b>(
        interaction: &'b ComponentInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
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

#[async_trait]
impl<'a> ModalSubmit<'a> for StandupCommand {
    async fn modal_submit<'b>(
        modal: &'b ModalInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> bool {
        modal.data.custom_id == "standups"
    }

    async fn handle_modal_submit<'b>(
        modal: &'b ModalInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        // send a simple message in the channel of the interaction WITH the data provided by the user
        let target_channel = modal.channel_id;

        if let Err(e) = target_channel
            .send_message(
                &context,
                CreateMessage::new().content(format!(
                    "{}: {:?}",
                    modal.member.as_ref().unwrap().user.name,
                    modal.data.components
                )),
            )
            .await
        {
            error!("Error sending message: {:?}", e);
        }

        Ok(CommandResponse::NoResponse)
    }
}
