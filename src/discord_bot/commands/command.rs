use serenity::{
    all::{AutocompleteOption, CommandInteraction, ComponentInteraction, ModalInteraction},
    async_trait,
    builder::{CreateAutocompleteResponse, CreateCommand},
    model::{application::CommandType, Permissions},
    prelude::Context,
};

use crate::{
    discord_bot::commands::{
        hide::HideCommand, ping::PingCommand, say::SayCommand, standup::StandupCommand,
    },
    state::AppState,
};

use super::util::CommandResponse;

const DEFAULT_PERMISSIONS: Permissions = Permissions::ADMINISTRATOR;

/// A command that can be used in a guild, restricted to administrators
#[async_trait]
pub trait Command<'a>: TryFrom<&'a CommandInteraction> {
    /// Get the name of the command
    fn name() -> &'static str;

    /// Get the description of the command
    fn description() -> &'static str;

    /// Get the discord defined usage of this command, to be sent to discord
    fn get_application_command_options(command: CreateCommand) -> CreateCommand;

    /// handle the execution of this application command
    async fn handle_application_command<'b>(
        self,
        interaction: &'b CommandInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse>;
}

/// A command that has support for autocomplete responses
#[async_trait]
pub trait AutocompleteCommand<'a>: Command<'a> {
    /// get the autocomplete options for this command, given the current input
    async fn autocomplete<'c>(
        command: &'c CommandInteraction,
        message: &'c AutocompleteOption,
        app_state: &'c AppState,
        context: &'c Context,
    ) -> Result<CreateAutocompleteResponse, CommandResponse>;
}

/// A command with a followup interaction component which must be handled
#[async_trait]
pub trait InteractionCommand<'a>: Command<'a> {
    /// validate if this message is related to a given command
    async fn answerable<'b>(
        interaction: &'b ComponentInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> bool;

    /// handle the generated interaction for this command
    async fn interaction<'b>(
        interaction: &'b ComponentInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse>;
}

#[async_trait]
pub trait ModalSubmit<'a>: Command<'a> + InteractionCommand<'a> {
    /// check if this modal submission is FOR this command
    async fn modal_submit<'b>(
        modal: &'b ModalInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> bool;

    /// handle the modal submit for this command
    async fn handle_modal_submit<'b>(
        modal: &'b ModalInteraction,
        app_state: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse>;
}

// #[async_trait]
// pub trait PaginatedResponse<'a>: Command<'a> {
//     /// Get the number of pages this response has
//     fn get_page_count(&self) -> usize;

//     /// Get a specific page of this response
//     fn get_page(&self, page: usize) -> CommandResponse<'a>;
// }

/// match against a list of provided command types, and generate an application command that can be registered with discord
macro_rules! application_command {
    ( $base:expr, $( $x:ty ),* $(,)? ) => {
        {
            /// ensures that the provided type has relevant traits
            fn assert_command<'a, T: Command<'a, Error=String>>() {}
            $(
                assert_command::<$x>();
                let mut v_base = <$x>::get_application_command_options(CreateCommand::new("unnamed command"));
                v_base = v_base
                    .name(<$x>::name())
                    .description(<$x>::description())
                    .default_member_permissions(DEFAULT_PERMISSIONS)
                    .dm_permission(false)
                    .kind(CommandType::ChatInput);
                $base.push(v_base);
            )*
        }
    };
}

/// match against a list of provided command types, and produce a response which can be sent to the user
macro_rules! command {
    ( $cmd:expr, $state:expr, $context:expr, $( $x:ty ),* $(,)? ) => {
        {
            /// ensures that the provided type has relevant traits
            fn assert_command<'a, T: Command<'a, Error=String>>() {}
            $(
                assert_command::<$x>();
                if ($cmd).data.name == <$x>::name() {
                    if let Ok(value) = <$x>::try_from($cmd) {
                        return value.handle_application_command($cmd, $state, $context).await
                    }
                }
            )*
            Err(CommandResponse::InternalFailure(String::from("Unsupported Command")))
        }
    };
}

/// match against a list of provided autocomplete command types, and produce a response which can be sent to the user
// macro_rules! autocomplete {
//     ( $cmd:expr, $state:expr, $context:expr, $( $x:ty ),* $(,)? ) => {
//         {
//             /// ensures that the provided type has relevant traits
//             fn assert_autocomplete<'a, T: AutocompleteCommand<'a, Error=String>>() {}
//             $(
//                 assert_autocomplete::<$x>();
//                 if ($cmd).data.name == <$x>::name() {
//                     return match $cmd.data.autocomplete() {
//                         Some(data) => <$x>::autocomplete($cmd, &data, $state, $context).await,
//                         None => Err(CommandResponse::InternalFailure(String::from("No Autocomplete Data Provided")))
//                     }
//                 }
//             )*
//             Err(CommandResponse::InternalFailure(String::from("Unsupported Autocomplete Command")))
//         }
//     };
// }

/// match against a list of provided interaction command types, and produce a response which can be sent to the user
macro_rules! interaction {
    ( $cmd:expr, $state:expr, $context:expr, $( $x:ty ),* $(,)? ) => {
        {
            /// ensures that the provided type has relevant traits
            fn assert_interaction<'a, T: InteractionCommand<'a, Error=String>>() {}
            $(
                assert_interaction::<$x>();
                if <$x>::answerable($cmd, $state, $context).await {
                    return <$x>::interaction($cmd, $state, $context).await
                }
            )*
            Err(CommandResponse::InternalFailure(String::from("Unsupported Interaction Command")))
        }
    };
}

/// match against a list of provided modal submit command types, and produce a response which can be sent to the user
macro_rules! modal {
    ( $cmd:expr, $state:expr, $context:expr, $( $x:ty ),* $(,)? ) => {
        {
            /// ensures that the provided type has relevant traits
            fn assert_modal<'a, T: ModalSubmit<'a, Error=String>>() {}
            $(
                assert_modal::<$x>();
                if <$x>::modal_submit($cmd, $state, $context).await {
                    return <$x>::handle_modal_submit($cmd, $state, $context).await
                }
            )*
            Err(CommandResponse::InternalFailure(String::from("Unsupported Modal Submit Command")))
        }
    };
}

pub fn application_command() -> Vec<CreateCommand> {
    let mut base = vec![];
    application_command!(
        &mut base,
        HideCommand,
        PingCommand,
        SayCommand,
        StandupCommand,
    );
    base
}

pub async fn command<'a>(
    command: &'a CommandInteraction,
    app_state: &'a AppState,
    context: &'a Context,
) -> Result<CommandResponse, CommandResponse> {
    command!(
        command,
        app_state,
        context,
        HideCommand,
        PingCommand,
        SayCommand,
        StandupCommand,
    )
}

#[allow(dead_code, unused_variables)]
pub async fn autocomplete<'a>(
    command: &'a CommandInteraction,
    app_state: &'a AppState,
    context: &'a Context,
) -> Result<CreateAutocompleteResponse, CommandResponse> {
    // autocomplete!(command, app_state, context)
    Err(CommandResponse::InternalFailure(String::from(
        "Unsupported Autocomplete Command",
    )))
}

pub async fn interaction<'a>(
    command: &'a ComponentInteraction,
    app_state: &'a AppState,
    context: &'a Context,
) -> Result<CommandResponse, CommandResponse> {
    interaction!(command, app_state, context, StandupCommand,)
}

pub async fn handle_modal<'a>(
    modal: &'a ModalInteraction,
    app_state: &'a AppState,
    context: &'a Context,
) -> Result<CommandResponse, CommandResponse> {
    modal!(modal, app_state, context, StandupCommand,)
}
