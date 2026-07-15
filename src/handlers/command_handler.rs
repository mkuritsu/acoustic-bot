use serenity::{
    all::{
        Color, Command, CommandInteraction, Context, CreateCommand, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Interaction,
        Ready,
    },
    async_trait,
};

use crate::commands::{clear_queue, play, queue, skip};

pub type CommandCreateFn = fn() -> CreateCommand;

const COMMAND_BUILDERS: &[(&str, CommandCreateFn)] = &[
    ("play", play::create),
    ("queue", queue::create),
    ("skip", skip::create),
    ("clearqueue", clear_queue::create),
];

pub struct CommandHandler;

impl CommandHandler {
    async fn handle_command(&self, ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
        match cmd.data.name.as_str() {
            "play" => play::execute(ctx, cmd).await,
            "queue" => queue::execute(ctx, cmd).await,
            "skip" => skip::execute(ctx, cmd).await,
            "clearqueue" => clear_queue::execute(ctx, cmd).await,
            _ => {
                panic!(
                    "handle_command called with unknown command: {}",
                    cmd.data.name
                );
            }
        }
    }
}

#[async_trait]
impl EventHandler for CommandHandler {
    async fn ready(&self, ctx: Context, _: Ready) {
        let commands = COMMAND_BUILDERS
            .iter()
            .map(|(_, builder)| builder())
            .collect::<Vec<_>>();
        if let Err(e) = Command::set_global_commands(&ctx.http, commands).await {
            panic!("failed to set global commands: {e:?}");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(cmd) = &interaction else {
            return;
        };

        if let Err(e) = self.handle_command(&ctx, cmd).await {
            let embed = CreateEmbed::new()
                .title("ERROR")
                .description(e.to_string())
                .color(Color::RED);

            let _ = cmd
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().add_embed(embed),
                    ),
                )
                .await;
        }
    }
}
