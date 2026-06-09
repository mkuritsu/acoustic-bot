use serenity::{
    all::{
        Color, Command, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage, EventHandler, Interaction, Ready,
    },
    async_trait,
};
use tracing::{error, info};

mod play;

pub struct CommandHandler;

#[async_trait]
impl EventHandler for CommandHandler {
    async fn ready(&self, ctx: Context, _: Ready) {
        let play_command = play::create();
        let commands = vec![play_command];
        if let Err(e) = Command::set_global_commands(&ctx.http, commands).await {
            error!("failed to set global commands: {e:?}");
        }
        info!("registered all global commands!");
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

impl CommandHandler {
    async fn handle_command(&self, ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
        match cmd.data.name.as_str() {
            "play" => play::execute(ctx, cmd).await?,
            _ => {
                error!("Unknown command: {}", cmd.data.name);
            }
        }
        Ok(())
    }
}
