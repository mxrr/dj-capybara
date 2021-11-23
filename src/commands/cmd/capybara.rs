use crate::commands::{Command, text_response};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand, CreateInteractionResponseData};
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::Error;

pub struct Capybara;

const CAPYBARA_GIFS: [&str; 1] = [
  "https://tenor.com/view/capybara-bucket-sit-spa-capybara-bucket-gif-23305453"
];

#[async_trait]
impl Command for Capybara {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    text_response(ctx, command, format!("{}", CAPYBARA_GIFS[0])).await
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("capybara")
      .description("Post a random capybara gif")
  }

}