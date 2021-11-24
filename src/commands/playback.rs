use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::GuildId;
use songbird::input::{Restartable, Input};
use tracing::{error};

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
    match Restartable::ytdl(param, true).await {
      Ok(s) => Ok(s.into()),
      Err(e) => {
        error!("Error fetching music file: {}", e);
        Err("Invalid URL".to_string())
      }
    }
  } else {
    match Restartable::ytdl_search(param.clone(), true).await {
      Ok(s) => Ok(s.into()),
      Err(e) => {
        error!("Error finding youtube video: {}", e);
        Err(format!("Nothing found with \"{}\"", param))
      }
    }
  }
}