use crate::commands::{text_response, Command};
use crate::constants::EMBED_COLOUR;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::Error;

pub struct Me;

#[async_trait]
impl Command for Me {
  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    if let Some(avatar) = ctx.cache.current_user().avatar_url() {
      match command
        .edit_original_interaction_response(&ctx.http, |response| {
          response.embed(|embed| {
            embed
              .image(avatar)
              .colour(EMBED_COLOUR)
              .footer(|footer| footer.text("💩"))
          })
        })
        .await
      {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
      }
    } else {
      text_response(ctx, command, "🍊").await
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("me").description("🍊")
  }
}
