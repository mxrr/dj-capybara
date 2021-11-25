use crate::commands::{Command, text_response};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::Error;
use crate::constants::EMBED_COLOUR;

pub struct Me;

#[async_trait]
impl Command for Me {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    if let Some(avatar) = ctx.cache.current_user().await.avatar_url() {
      match command
        .edit_original_interaction_response(&ctx.http, |response| {
          response
            .create_embed(|embed| {
              embed
                .image(avatar)
                .colour(EMBED_COLOUR)
                .footer(|footer| {
                  footer
                    .text("💩")
                })
            })
        }).await
        {
          Ok(_) => Ok(()),
          Err(e) => Err(e),
        }
    } else {
      text_response(ctx, command, "🍊".to_string()).await
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("me")
      .description("🍊")
  }

}