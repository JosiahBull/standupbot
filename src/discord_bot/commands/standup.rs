use log::error;
use serenity::{
    all::{ActionRowComponent, CommandInteraction, ComponentInteraction, ModalInteraction},
    async_trait,
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateInputText, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateModal,
    },
    prelude::Context,
};

use crate::state::AppState;

use super::{
    command::{Command, InteractionCommand, ModalSubmit},
    util::CommandResponse,
};

static QUESTION_LIST: &[&str] = &[
    "What did you work on last week?",
    "What are you working on this week?",
    "When are you aiming to finish?",
    "Is there anything blocking you?",
    "is there anything you need help with?",
];

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
        "Run the weekly standups"
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
        _: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        interaction
            .create_response(
                context,
                CreateInteractionResponse::Modal(
                    CreateModal::new("standups", "Standups").components({
                        QUESTION_LIST
                            .iter()
                            .enumerate()
                            .map(|(i, question)| {
                                CreateActionRow::InputText(CreateInputText::new(
                                    serenity::all::InputTextStyle::Short,
                                    *question,
                                    format!("standup-question-{}", i),
                                ))
                            })
                            .collect()
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
        _: &'b AppState,
        _: &'b Context,
    ) -> bool {
        modal.data.custom_id == "standups"
    }

    async fn handle_modal_submit<'b>(
        modal: &'b ModalInteraction,
        _: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        // send a simple message in the channel of the interaction WITH the data provided by the user
        let username = &modal.user.name;

        let mut result_strings = vec![];

        for (i, question) in QUESTION_LIST.iter().enumerate() {
            let answer = {
                match &modal.data.components[i].components[0] {
                    ActionRowComponent::InputText(input) => input,
                    _ => panic!("Unexpected component type"),
                }
            };

            result_strings.push(format!("**{}**\r{}", question, answer.value));
        }

        // if the total length of all result strings is >=1900 chars, throw an error
        let char_count = result_strings.iter().map(|s| s.len()).sum::<usize>();

        if char_count >= 1900 {
            if let Err(e) = modal.create_response(&context,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("Your standup is too long, please shorten it")
                    .ephemeral(true)
                )

            ).await {
                error!("Error sending standup response: {}", e);
                return Ok(CommandResponse::NoResponse)
            }
        }

        if let Err(e) = modal
            .create_response(
                &context,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(format!(
                        "**Standup Submission by {}:**\r{}",
                        username,
                        result_strings.join("\r")
                    )),
                ),
            )
            .await
        {
            error!("Error sending standup response: {}", e);
        }

        Ok(CommandResponse::NoResponse)
    }
}
