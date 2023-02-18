use crate::commands::Command;
use crate::constants::EMBED_COLOUR;
use chrono::prelude::*;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::Error;

pub struct Capybara;

const FILE_URL: &str = "https://karei.dev/files/capybara-gifs/";
const FILE_PREFIX: &str = "cp_";

#[async_trait]
impl Command for Capybara {
  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
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
      .edit_original_interaction_response(&ctx.http, |response| {
        response.embed(|embed| {
          embed
            .image(format!(
              "{url}{prefix}{filename}.gif",
              url = FILE_URL,
              prefix = FILE_PREFIX,
              filename = filename
            ))
            .colour(EMBED_COLOUR)
        })
      })
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e),
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("capybara")
      .description("Post today's capybara gif")
  }
}
