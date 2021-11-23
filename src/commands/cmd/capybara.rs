use crate::commands::{Command};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;

pub struct Capybara;

const CAPYBARA_GIFS: [&str; 1] = [
  "https://tenor.com/view/capybara-bucket-sit-spa-capybara-bucket-gif-23305453"
];

#[async_trait]
impl Command for Capybara {

  async fn execute(_ctx: &Context, _command: &ApplicationCommandInteraction) -> String {
    format!("{}", CAPYBARA_GIFS[0])
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("capybara")
      .description("Post a random capybara gif")
  }

}