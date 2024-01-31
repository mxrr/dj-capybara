use crate::commands::{text_response, Command};
use crate::constants::EMBED_COLOUR;
use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateEmbed, CreateEmbedFooter, EditInteractionResponse};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::Error;

pub struct Me;

#[async_trait]
impl Command for Me {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let avatar = ctx.cache.current_user().avatar_url();
    if let Some(avatar) = avatar {
      match command
        .edit_response(
          &ctx.http,
          EditInteractionResponse::new().embed(
            CreateEmbed::new()
              .colour(EMBED_COLOUR)
              .image(avatar)
              .footer(CreateEmbedFooter::new("💩")),
          ),
        )
        .await
      {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
      }
    } else {
      text_response(ctx, command, "🍊").await
    }
  }

  fn name() -> &'static str {
    "me"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("🍊")
  }
}
