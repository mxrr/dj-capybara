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
use crate::constants::EMBED_COLOUR;

mod cmd;
mod playback;
mod utils;

static COMMAND_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(3);


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
      Ok(c) => {
        let cmd_list = c
          .iter()
          .fold("".to_string(), |mut a, c| {
            let s = format!("{}\n", c.name);
            a.push_str(&s);
            a
          });
        info!("Added commands for Guild({}):\n{}", guild, cmd_list)
      },
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
    .create_application_command(cmd::Seek::info)
    .create_application_command(cmd::Skip::info)
    .create_application_command(cmd::Queue::info)
    .create_application_command(cmd::Me::info)
    .create_application_command(cmd::Info::info)
    .create_application_command(cmd::Stop::info)
    .create_application_command(cmd::Eval::info)
}

pub async fn handle_commands(ctx: &Context, command: ApplicationCommandInteraction) {
  let name = command.data.name.clone();
  let user = command.user.clone();
  match command.create_interaction_response(&ctx.http, |response| {
    response
      .kind(InteractionResponseType::DeferredChannelMessageWithSource)
      .interaction_response_data(|message| {
        message.content("Loading".to_string())
      })
  }).await {
    Ok(_) => info!("{} command deferred", name),
    Err(e) => error!("Error deferring command {}: {}",name , e),
  }

  let command_copy = command.clone();
  let result = match name.as_str(){
    "join" => cmd::Join::execute(ctx, command_copy),
    "leave" => cmd::Leave::execute(ctx, command_copy),
    "play" => cmd::Play::execute(ctx, command_copy),
    "seek" => cmd::Seek::execute(ctx, command_copy),
    "skip" => cmd::Skip::execute(ctx, command_copy),
    "queue" => cmd::Queue::execute(ctx, command_copy),
    "stop" => cmd::Stop::execute(ctx, command_copy),
    "capybara" => cmd::Capybara::execute(ctx, command_copy),
    "me" => cmd::Me::execute(ctx, command_copy),
    "info" => cmd::Info::execute(ctx, command_copy),
    "eval" => cmd::Eval::execute(ctx, command_copy),
    _ => Box::pin(text_response(ctx, command_copy, "Invalid command"))
  };

  match tokio::time::timeout(COMMAND_TIMEOUT, result).await {
    Ok(result) => {
      if let Err(e) = result {
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
    },
    Err(e) => {
      error!("Couldn't respond to command: {}", e);
      error!("{user} failed running command {cmd}", 
        user = user.tag(),
        cmd = name
      );
      text_response(ctx, command, "Took too long processing command").await.unwrap_or(());
    }
  }
}


pub async fn text_response<D>(ctx: &Context, command: ApplicationCommandInteraction, text: D) -> Result<(), Error>
where D: ToString, {
  match command
    .edit_original_interaction_response(&ctx.http, |response| {
      response
        .create_embed(|embed| {
          embed
            .title(text)
            .colour(EMBED_COLOUR)
        })
    }).await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e),
    }
}

