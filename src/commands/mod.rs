use crate::config::{ConfigStorage};
use serenity::model::id::ChannelId;
use serenity::model::interactions::{
  InteractionResponseType,
  application_command::ApplicationCommandInteraction
};
use serenity::prelude::Context;
use serenity::model::prelude::{Ready, GuildId};
use tracing::{info, error};

pub async fn register_commands(ctx: &Context, ready: &Ready) {
  let config_lock = {
    let data = ctx.data.read().await;
    data.get::<ConfigStorage>()
      .expect("No config in global storage")
      .clone()
  };

  if let Some(guild) = config_lock.guild_id {
    let commands = guild.set_application_commands(&ctx.http, |commands| {
      commands
        .create_application_command(|command| {
          command.name("join").description("Join current voice channel")
        })
        .create_application_command(|command| {
          command.name("leave").description("Leave voice channel")
        })
    })
    .await;

    match commands {
      Ok(c) => info!("Added commands for Guild({}): {:#?}", guild, c),
      Err(e) => panic!("Couldn't set application commands: {:#?}", e)
    }
    
    
  } else {
    unimplemented!("Global commands")
  }
}

pub async fn handle_commands(ctx: &Context, command: ApplicationCommandInteraction) {
  let content = match command.data.name.as_str() {
    "join" => join(ctx, &command).await,
    "leave" => leave(ctx, &command).await,
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
      error!("Couldn't respond to command: {}", e)
    }
}


struct VOIPData {
  pub channel_id: ChannelId,
  pub guild_id: GuildId
}

async fn get_voip_data(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<VOIPData, String> {
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
          return Err("Error getting voice channel".to_string())
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


async fn join(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
  let voip_data = match get_voip_data(ctx, command).await {
    Ok(v) => v,
    Err(s) => return s
  };

  let guild_id = voip_data.guild_id;
  let channel_id = voip_data.channel_id;

  let manager = match songbird::get(ctx).await {
    Some(arc) => arc.clone(),
    None => {
      error!("Error with songbird client");
      return "Error getting voice client".to_string()
    }
  };

  let _handler = manager.join(guild_id, channel_id).await;

  if let Some(channel_name) = channel_id.name(&ctx.cache).await {
    format!("Joined channel {}", channel_name)
  } else {
    "Joined channel".to_string()
  }
}

async fn leave(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
  let voip_data = match get_voip_data(ctx, command).await {
    Ok(v) => v,
    Err(s) => return s
  };

  let manager = match songbird::get(ctx).await {
    Some(arc) => arc.clone(),
    None => {
      error!("Error with songbird client");
      return "Error getting voice client".to_string()
    }
  };

  let guild_id = voip_data.guild_id;

  if manager.get(guild_id).is_some() {
    if let Err(e) = manager.remove(guild_id).await {
      error!("Error leaving voice channel: {}", e);
      return "Error leaving channel".to_string()
    } else {
      return "Left channel".to_string()
    }
  } else {
    "Not in a voice channel".to_string()
  }
}