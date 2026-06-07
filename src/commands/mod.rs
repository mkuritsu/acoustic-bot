use serenity::{
    all::{Command, Context, EventHandler, Interaction, Ready},
    async_trait,
};
use tracing::error;

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
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = &interaction else {
            return;
        };
        if &command.data.name[..] == "play" {
            play::execute(&ctx, command).await;
        }
    }
}
