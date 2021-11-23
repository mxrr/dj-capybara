use crate::config::{ConfigStorage};
use serenity::builder::{CreateApplicationCommands, CreateApplicationCommand};
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

pub mod queue;

mod cmd;
mod playback;


#[async_trait]
trait Command {
  async fn execute(ctx: &Context, command: &ApplicationCommandInteraction) -> String;
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
  let content = match command.data.name.as_str() {
    "join" => cmd::Join::execute(ctx, &command).await,
    "leave" => cmd::Leave::execute(ctx, &command).await,
    "play" => cmd::Play::execute(ctx, &command).await,
    "capybara" => cmd::Capybara::execute(ctx, &command).await,
    _ => "Invalid command".to_string()
  };

  if let Err(e) = command
    .create_interaction_response(&ctx.http, |response| {
      response
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|message| message.content(content))
    })
    .await
    {
      error!("Couldn't respond to command: {}", e);
      error!("{name} failed running command {cmd}", 
        name = command.user.tag(),
        cmd = command.data.name
      )
    }
    else 
    {
      info!("{name} ran command {cmd}", 
        name = command.user.tag(),
        cmd = command.data.name
      )
    }
}

