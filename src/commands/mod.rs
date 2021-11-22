use crate::config::{ConfigStorage};
use serenity::builder::CreateApplicationCommands;
use serenity::model::id::ChannelId;
use serenity::model::interactions::{
  InteractionResponseType,
  application_command::{
    ApplicationCommandInteraction, 
    ApplicationCommandOptionType,
    ApplicationCommandInteractionDataOptionValue
  }
};
use serenity::prelude::Context;
use serenity::model::prelude::{Ready, GuildId};
use songbird::input::Input;
use tracing::{info, error};

pub mod queue;


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
    .create_application_command(|command| {
      command
        .name("join")
        .description("Join current voice channel")
    })
    .create_application_command(|command| {
      command
        .name("leave")
        .description("Leave voice channel")
    })
    .create_application_command(|command| {
      command
        .name("play")
        .description("Play a YouTube video or any music/video file")
        .create_option(|option| {
          option
            .name("search")
            .description("Search term or a link to a YouTube video or a file")
            .kind(ApplicationCommandOptionType::String)
            .required(true)
        })
    })
}

pub async fn handle_commands(ctx: &Context, command: ApplicationCommandInteraction) -> bool {
  let content = match command.data.name.as_str() {
    "join" => join(ctx, &command).await,
    "leave" => leave(ctx, &command).await,
    "play" => play(ctx, &command).await,
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


struct VOIPData {
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
}

async fn join(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
  let voip_data = match VOIPData::from(ctx, command).await {
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
  let voip_data = match VOIPData::from(ctx, command).await {
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

async fn get_source(param: String) -> Result<Input, String> {
  if param.contains("https://") {
    match songbird::ytdl(param).await {
      Ok(s) => Ok(s),
      Err(e) => {
        error!("Error fetching music file: {}", e);
        Err("Invalid URL".to_string())
      }
    }
  } else {
    match songbird::input::ytdl_search(param).await {
      Ok(s) => Ok(s),
      Err(e) => {
        error!("Error finding youtube video: {}", e);
        Err("Nothing found".to_string())
      }
    }
  }
}

async fn play(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
  let voip_data_f = VOIPData::from(ctx, command);

  let option = match command.data.options.get(0) {
    Some(o) => {
      match o.resolved.as_ref() {
        Some(opt_val) => opt_val.clone(),
        None => {
          error!("No options provided");
          return "No search term or URL in request".to_string()
        }
      }
    },
    None => {
      error!("No options provided");
      return "No search term or URL in request".to_string()
    }
  };

  let param = if let ApplicationCommandInteractionDataOptionValue::String(s) = option {
    s
  } else {
    error!("Empty URL provided");
    return "No search term or URL in request".to_string()
  };

  let voip_data = match voip_data_f.await {
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

  let handler = match manager.get(guild_id) {
    Some(h) => h,
    None => {
      let join = manager.join(guild_id, channel_id).await;
      match join.1 {
        Ok(_) => join.0,
        Err(e) => {
          error!("Error joining voice channel: {}", e);
          return "Not in a voice channel".to_string()
        }
      }
    }
  };

  let source = match get_source(param).await {
    Ok(s) => s,
    Err(s) => return s,
  };

  let title = source
    .metadata
    .title
    .clone()
    .unwrap_or("Missing title".to_string());

  let artist = source
    .metadata
    .artist
    .clone()
    .unwrap_or("Missing artist".to_string());

  let mut handler_lock = handler.lock().await;
  let _handle = handler_lock.play_only_source(source);

  format!("Playing \"{}\" - {}", title, artist)
}