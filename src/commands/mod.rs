use crate::config::{ConfigStorage};
use serenity::builder::{
  CreateApplicationCommand, 
  CreateApplicationCommands
};
use serenity::model::interactions::{
  InteractionResponseType,
  application_command::{
    ApplicationCommandInteraction,
  }
};
use serenity::prelude::Context;
use serenity::model::prelude::Ready;
use tracing::{info, error};
use serenity::async_trait;
use serenity::Error;

pub mod queue;

mod cmd;
mod playback;


#[async_trait]
trait Command {
  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error>;
  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

pub async fn register_commands(ctx: &Context, _ready: &Ready) {
  let config_lock = {
    let data = ctx.data.read().await;
    data.get::<ConfigStorage>()
      .expect("No config in global storage")
      .clone()
  };

  if let Some(guild) = config_lock.guild_id {
    let commands = guild
      .set_application_commands(&ctx.http, command_list)
      .await;

    match commands {
      Ok(c) => info!("Added commands for Guild({}): {:#?}", guild, c),
      Err(e) => panic!("Couldn't set application commands: {:#?}", e)
    }
    
    
  } else {
    unimplemented!("Global commands")
  }
}

fn command_list(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
  commands
    .create_application_command(cmd::Join::info)
    .create_application_command(cmd::Leave::info)
    .create_application_command(cmd::Play::info)
    .create_application_command(cmd::Capybara::info)
}

pub async fn handle_commands(ctx: &Context, command: ApplicationCommandInteraction) {
  let name = command.data.name.clone();
  let user = command.user.clone();
  let result = match name.as_str(){
    "join" => cmd::Join::execute(ctx, command),
    "leave" => cmd::Leave::execute(ctx, command),
    "play" => cmd::Play::execute(ctx, command),
    "capybara" => cmd::Capybara::execute(ctx, command),
    _ => Box::pin(text_response(ctx, command, format!("Invalid command")))
  };

  if let Err(e) = result.await {
    error!("Couldn't respond to command: {}", e);
    error!("{user} failed running command {cmd}", 
      user = user.tag(),
      cmd = name
    )
  } else {
    info!("{user} ran command {cmd}", 
      user = user.tag(),
      cmd = name
    )
  }
}


pub async fn text_response(ctx: &Context, command: ApplicationCommandInteraction, text: String) -> Result<(), Error> {
  command
    .create_interaction_response(&ctx.http, |response| {
      response
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|message| {
          message.content(text)
        })
    }).await
}

