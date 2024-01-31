use crate::commands::Command;
use crate::constants::EMBED_COLOUR;
use chrono::prelude::*;
use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateEmbed, EditInteractionResponse};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::Error;

pub struct Capybara;

const FILE_URL: &str = "https://karei.dev/files/capybara-gifs/";
const FILE_PREFIX: &str = "cp_";

#[async_trait]
impl Command for Capybara {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let filename = match Local::now().weekday() {
      Weekday::Mon => "monday",
      Weekday::Tue => "tuesday",
      Weekday::Wed => "wednesday",
      Weekday::Thu => "thursday",
      Weekday::Fri => "friday",
      Weekday::Sat => "saturday",
      Weekday::Sun => "sunday",
    };
    match command
      .edit_response(
        &ctx.http,
        EditInteractionResponse::new().embed(
          CreateEmbed::new()
            .image(format!(
              "{url}{prefix}{filename}.gif",
              url = FILE_URL,
              prefix = FILE_PREFIX,
              filename = filename
            ))
            .colour(EMBED_COLOUR),
        ),
      )
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e),
    }
  }

  fn name() -> &'static str {
    "capybara"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("Post today's capybara gif")
  }
}
