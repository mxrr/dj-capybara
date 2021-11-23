use crate::config::{ConfigStorage};
use serenity::builder::{CreateApplicationCommands, CreateApplicationCommand};
use serenity::model::id::ChannelId;
use serenity::model::interactions::{
  InteractionResponseType,
  application_command::{
    ApplicationCommandInteraction,
  }
};
use serenity::prelude::Context;
use serenity::model::prelude::{Ready, GuildId};
use songbird::input::Input;
use tracing::{info, error};
use serenity::async_trait;

pub mod queue;

mod cmd;


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

pub async fn handle_commands(ctx: &Context, command: ApplicationCommandInteraction) -> bool {
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
      false
    }
    else 
    {
      true
    }
}


pub struct VOIPData {
  pub channel_id: ChannelId,
  pub guild_id: GuildId
}

impl VOIPData {
  pub async fn from(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<VOIPData, String> {
    let guild_from_command = command.guild_id;
    let guild_id = match guild_from_command {
      Some(g_id) => g_id,
      None => {
        error!("Error getting guild from command");
        return Err("Error getting guild information".to_string())
      }
    };
  
    let guild_cache = guild_id.to_guild_cached(&ctx.cache).await;
  
    let channel_id = match guild_cache {
      Some(guild) => {
        let ch = guild
          .voice_states
          .get(&command.member.as_ref().unwrap().user.id.clone())
          .and_then(|vs| vs.channel_id);
        
        match ch {
          Some(c) => c,
          None => {
            error!("Error getting channel id");
            return Err("Join a voice channel first".to_string())
          }
        }
      },
      None => {
        error!("Error getting guild from cache");
        return Err("Error getting guild information".to_string())
      }
    };
  
    let data = VOIPData{channel_id, guild_id};
    Ok(data)
  }
}


pub async fn get_source(param: String) -> Result<Input, String> {
  if param.contains("https://") {
    match songbird::ytdl(param).await {
      Ok(s) => Ok(s),
      Err(e) => {
        error!("Error fetching music file: {}", e);
        Err("Invalid URL".to_string())
      }
    }
  } else {
    match songbird::input::ytdl_search(param.clone()).await {
      Ok(s) => Ok(s),
      Err(e) => {
        error!("Error finding youtube video: {}", e);
        Err(format!("Nothing found with \"{}\"", param))
      }
    }
  }
}

