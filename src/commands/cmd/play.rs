use std::sync::Arc;

use crate::commands::{
  playback::{
    format_duration, format_duration_live, get_queue_length_and_duration, get_source, SongMetadata,
    SongMetadataKey, VOIPData,
  },
  text_response,
  utils::remove_md_characters,
  Command,
};
use crate::constants::EMBED_COLOUR;
use serenity::{
  all::ResolvedValue,
  async_trait,
  builder::{
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
    CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, EditInteractionResponse,
  },
  client::Context,
  model::application::{CommandInteraction, CommandOptionType},
  model::id::{ChannelId, GuildId},
  prelude::Mutex,
  Error,
};
use songbird::{events::Event, Call, EventContext, EventHandler, Songbird, TrackEvent};
use tracing::error;

pub struct Play;

const PARAM_OPTION_NAME: &str = "search";

#[async_trait]
impl Command for Play {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let param = match command
      .data
      .options()
      .iter()
      .find(|o| o.name == PARAM_OPTION_NAME)
    {
      Some(o) => {
        if let ResolvedValue::String(s) = o.value {
          s.to_string()
        } else {
          error!("Invalid search option provided");
          return text_response(ctx, command, "No search term or URL in request").await;
        }
      }
      None => {
        error!("No options provided");
        return text_response(ctx, command, "No search term or URL in request").await;
      }
    };

    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await,
    };

    let guild_id = voip_data.guild_id;

    let http_client = {
      let data = ctx.data.read().await;
      data
        .get::<crate::constants::HttpKey>()
        .cloned()
        .expect("HttpClient did not exist")
    };

    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client").await;
      }
    };

    let handler_lock = match manager.get(guild_id) {
      Some(h) => {
        if voip_data.compare_to_call(&h).await {
          h
        } else {
          match join_channel(manager, voip_data).await {
            Ok(h) => h,
            Err(e) => return text_response(ctx, command, e).await,
          }
        }
      }
      None => match join_channel(manager, voip_data).await {
        Ok(h) => h,
        Err(e) => return text_response(ctx, command, e).await,
      },
    };

    let mut source = get_source(http_client, param);
    let metadata = SongMetadata::from_source(&mut source).await;

    let mut handler = handler_lock.lock().await;

    let handle = handler.enqueue_input(source.into()).await;
    {
      let mut data = handle.typemap().write().await;
      data.insert::<SongMetadataKey>(metadata.clone());
    }
    match handle.add_event(
      Event::Track(TrackEvent::Error),
      SongError {
        ctx: ctx.clone(),
        command: command.clone(),
      },
    ) {
      Ok(_) => (),
      Err(e) => error!("Error adding SongError event: {}", e),
    }
    let embed_title = match handler.queue().len() == 1 {
      true => "Playing",
      false => "Added to queue",
    };

    if handler.queue().is_empty() {
      return text_response(ctx, command, "Error playing song").await;
    }

    if handler.queue().len() > 1 {
      match handle.add_event(
        Event::Track(TrackEvent::Play),
        SongStart {
          channel_id: command.channel_id,
          guild_id,
          ctx: ctx.clone(),
        },
      ) {
        Ok(_) => (),
        Err(e) => error!("Error adding SongStart event: {}", e),
      }
    }

    let url = metadata.url.clone().unwrap_or_default();
    let (count, duration) = get_queue_length_and_duration(&handler.queue().current_queue()).await;

    let user_nick = remove_md_characters(
      command
        .user
        .nick_in(&ctx.http, guild_id)
        .await
        .unwrap_or_else(|| command.user.tag()),
    );

    match command
      .edit_response(
        &ctx.http,
        EditInteractionResponse::new()
          .embed(
            CreateEmbed::new()
              .title(embed_title)
              .image(metadata.thumbnail)
              .author(CreateEmbedAuthor::new(user_nick).icon_url(command.user.face()))
              .colour(EMBED_COLOUR)
              .fields(vec![
                ("Track", remove_md_characters(metadata.title.clone()), true),
                (
                  "Duration",
                  format_duration_live(metadata.duration, &metadata.title).to_string(),
                  true,
                ),
              ])
              .footer(CreateEmbedFooter::new(format!(
                "{} songs in queue - {}",
                count,
                format_duration(duration)
              ))),
          )
          .components(vec![CreateActionRow::Buttons(vec![
            CreateButton::new_link(url).label("Open in browser"),
          ])]),
      )
      .await
    {
      Ok(_m) => Ok(()),
      Err(e) => Err(e),
    }
  }

  fn name() -> &'static str {
    "play"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name())
      .description("Play a YouTube video or any music/video file")
      .add_option(
        CreateCommandOption::new(
          CommandOptionType::String,
          PARAM_OPTION_NAME,
          "Search term or a link to a Youtube video or a file",
        )
        .required(true),
      )
  }
}

struct SongStart {
  channel_id: ChannelId,
  guild_id: GuildId,
  ctx: Context,
}

#[async_trait]
impl EventHandler for SongStart {
  async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
    let handle = if let EventContext::Track(track_ctx) = ctx {
      let (_state, handle) = track_ctx[0];
      handle
    } else {
      return Some(Event::Cancel);
    };

    let metadata = SongMetadata::from_handle(handle).await;

    let manager = match songbird::get(&self.ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return Some(Event::Cancel);
      }
    };

    let handler_lock = match manager.get(self.guild_id) {
      Some(h) => h,
      None => {
        error!("Error locking guild voice client");
        return Some(Event::Cancel);
      }
    };

    let handler = handler_lock.lock().await;

    let (count, duration) = get_queue_length_and_duration(&handler.queue().current_queue()).await;

    drop(handler);
    let url = metadata.url.clone().unwrap_or_default();

    match self
      .channel_id
      .send_message(
        &self.ctx.http,
        CreateMessage::new()
          .embed(
            CreateEmbed::new()
              .title("Playing")
              .colour(EMBED_COLOUR)
              .image(metadata.thumbnail)
              .fields(vec![
                ("Track", remove_md_characters(metadata.title.clone()), true),
                (
                  "Duration",
                  format_duration_live(metadata.duration, &metadata.title).to_string(),
                  true,
                ),
              ])
              .footer(CreateEmbedFooter::new(format!(
                "{} songs in queue - {}",
                count,
                format_duration(duration)
              ))),
          )
          .components(vec![CreateActionRow::Buttons(vec![
            CreateButton::new_link(url).label("Open in browser"),
          ])]),
      )
      .await
    {
      Ok(_o) => return None,
      Err(e) => {
        error!("{}", e);
        return None;
      }
    }
  }
}

struct SongError {
  pub command: CommandInteraction,
  pub ctx: Context,
}

#[async_trait]
impl EventHandler for SongError {
  async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
    match text_response(&self.ctx, &self.command, "Error playing song").await {
      Ok(_) => None,
      Err(e) => {
        error!("Failed editing error response: {}", e);
        None
      }
    }
  }
}

async fn join_channel(
  manager: Arc<Songbird>,
  voip_data: VOIPData,
) -> Result<Arc<Mutex<Call>>, String> {
  let join = manager.join(voip_data.guild_id, voip_data.channel_id).await;
  match join {
    Ok(j) => Ok(j),
    Err(e) => {
      error!("Error joining voice channel: {}", e);
      Err("Not in a voice channel".to_string())
    }
  }
}
