use crate::commands::{
  Command, 
  playback::{
    SongMetadata, 
    VOIPData, 
    format_duration, 
    get_queue_length_and_duration, 
    get_source
  }, 
  text_response,
  utils::remove_md_characters,
};
use serenity::{async_trait, model::id::{ChannelId, GuildId}};
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::interactions::application_command::{
  ApplicationCommandInteraction,
  ApplicationCommandInteractionDataOptionValue,
  ApplicationCommandOptionType,
};
use tracing::error;
use serenity::Error;
use serenity::model::interactions::message_component::ButtonStyle;
use songbird::{EventContext, EventHandler, TrackEvent, events::Event};
use crate::constants::EMBED_COLOUR;
use std::time::Duration;

pub struct Play;

#[async_trait]
impl Command for Play {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {

    let option = match command.data.options.get(0) {
      Some(o) => {
        match o.resolved.as_ref() {
          Some(opt_val) => opt_val.clone(),
          None => {
            error!("No options provided");
            return text_response(ctx, command, "No search term or URL in request".to_string()).await
          }
        }
      },
      None => {
        error!("No options provided");
        return text_response(ctx, command, "No search term or URL in request".to_string()).await
      }
    };
  
    let param = if let ApplicationCommandInteractionDataOptionValue::String(s) = option {
      s
    } else {
      error!("Empty URL provided");
      return text_response(ctx, command, "No search term or URL in request".to_string()).await
    };
  
    let voip_data = match VOIPData::from(ctx, &command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await
    };
  
    let guild_id = voip_data.guild_id;
    let channel_id = voip_data.channel_id;
  
    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client".to_string()).await
      }
    };
  
    let handler_lock = match manager.get(guild_id) {
      Some(h) => h,
      None => {
        let join = manager.join(guild_id, channel_id).await;
        match join.1 {
          Ok(_) => join.0,
          Err(e) => {
            error!("Error joining voice channel: {}", e);
            return text_response(ctx, command, "Not in a voice channel".to_string()).await
          }
        }
      }
    };
  
    let source = match get_source(param).await {
      Ok(s) => s,
      Err(s) => return text_response(ctx, command, s).await,
    };

    let mut handler = handler_lock.lock().await;



    let (track, handle) = songbird::tracks::create_player(source);
    match handle.add_event(
      Event::Track(TrackEvent::End),
      SongEnd{
        channel_id: command.channel_id,
        guild_id: guild_id,
        ctx: ctx.clone(),
      }
    ) {
      Ok(_) => (),
      Err(e) => error!("Error adding track event: {}", e),
    }

    let metadata = SongMetadata::from_handle(handle);
    let url = metadata.url.clone().unwrap_or_default();
    let embed_title = match handler.queue().is_empty() {
      true => "Playing",
      false => "Added to queue",
    };


    handler.enqueue(track);

    let (count, duration) = get_queue_length_and_duration(
      &handler
      .queue()
      .current_queue()
    );

    let user_nick = remove_md_characters(
      command
        .user
        .nick_in(&ctx.http, guild_id)
        .await
        .unwrap_or(command.user.tag())
      );
  
    match command
      .edit_original_interaction_response(&ctx.http, |response| {
        response
          .create_embed(|embed| {
            embed
              .title(embed_title)
              .image(metadata.thumbnail)
              .author(|author| {
                author
                  .name(user_nick)
                  .icon_url(command.user.face())
              })
              .colour(EMBED_COLOUR)
              .fields(vec![
                ("Track", remove_md_characters(metadata.title), true),
                ("Duration", format_duration(metadata.duration), true),
              ])
              .footer(|footer| {
                footer
                  .text(format!("{} songs in queue - {}", count, format_duration(duration)))
              })
          })
          .components(|components| {
            components
              .create_action_row(|row| {
                row
                  .create_button(|button| {
                    button
                      .style(ButtonStyle::Link)
                      .label("Open in browser")
                      .url(url)
                  })
              })
          })

      }).await {
        Ok(_m) => Ok(()),
        Err(e) => Err(e)
      }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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
  }

}

struct SongEnd {
  channel_id: ChannelId,
  guild_id: GuildId,
  ctx: Context,
}

#[async_trait]
impl EventHandler for SongEnd {
  async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {

    if let EventContext::Track(track_ctx) = ctx {
      let (state, _handle) = track_ctx[0];

      if state.position == Duration::from_secs(0) {
        return Some(Event::Cancel)
      }

    } else {
      return Some(Event::Cancel)
    }


    let manager = match songbird::get(&self.ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return Some(Event::Cancel)
      }
    };
  
    let handler_lock = match manager.get(self.guild_id) {
      Some(h) => h,
      None => {
        error!("Error locking guild voice client");
        return Some(Event::Cancel)
      },
    };

    let handler = handler_lock.lock().await;

    if handler.queue().is_empty() {
      match self
        .channel_id
        .send_message(&self.ctx.http, |message| {
          message
            .embed(|embed| {
              embed
                .colour(EMBED_COLOUR)
                .title("End of queue")
            })
        })
        .await {
          Ok(_o) => {
            return None
          },
          Err(e) => {
            error!("{}", e);
            return None
          },
        }
    } else {
      let current = match handler.queue().current() {
        Some(t) => t,
        None => return None,
      };

      let metadata = SongMetadata::from_handle(current);

      let (count, duration) = get_queue_length_and_duration(
        &handler
        .queue()
        .current_queue()
      );

      match self
      .channel_id
      .send_message(&self.ctx.http, |message| {
        message
          .embed(|embed| {
            embed
              .title("Playing")
              .colour(EMBED_COLOUR)
              .image(metadata.thumbnail)
              .fields(vec![
                ("Track", metadata.title, true),
                ("Duration", format_duration(metadata.duration), true),
              ])
              .footer(|footer| {
                footer
                  .text(format!("{} songs in queue - {}", count, format_duration(duration)))
              })

          })
      })
      .await {
        Ok(_o) => {
          return None
        },
        Err(e) => {
          error!("{}", e);
          return None
        },
      }
    }
  }
}
