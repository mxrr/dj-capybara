use crate::commands::{
  Command, 
  text_response,
  playback::{
    VOIPData, 
    get_source,
  },
};
use serenity::{async_trait, model::interactions::InteractionResponseType};
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::interactions::application_command::{
  ApplicationCommandInteraction,
  ApplicationCommandInteractionDataOptionValue,
  ApplicationCommandOptionType,
};
use tracing::{error};
use serenity::Error;
use std::time::Duration;
use serenity::utils::Colour;
use serenity::model::interactions::message_component::ButtonStyle;

pub struct Play;

#[async_trait]
impl Command for Play {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    command.create_interaction_response(&ctx.http, |response| {
      response
        .kind(InteractionResponseType::DeferredChannelMessageWithSource)
        .interaction_response_data(|message| {
          message.content("Loading song".to_string())
        })
    });


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
  
    let handler = match manager.get(guild_id) {
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
  
    let title = source
      .metadata
      .title
      .clone()
      .unwrap_or("Missing title".to_string());
  
    let length = source
      .metadata
      .duration
      .clone()
      .unwrap_or(Duration::from_secs(0));

    let thumbnail = source
      .metadata
      .thumbnail
      .clone()
      .unwrap_or("https://mxrr.dev/files/christmas.gif".to_string());

    let url = source
      .metadata
      .source_url
      .clone()
      .unwrap_or("".to_string());
  
    let mut handler_lock = handler.lock().await;
    let _handle = handler_lock.play_only_source(source);
  
    command
      .create_interaction_response(&ctx.http, |response| {
        response
          .kind(InteractionResponseType::ChannelMessageWithSource)
          .interaction_response_data(|message| {
            message
              .create_embed(|embed| {
                embed
                  .image(thumbnail)
                  .colour(Colour::from_rgb(232, 12, 116))
                  .fields(vec![
                    ("Track", title, true),
                    ("Length", format!("{:?}", length), true),
                    ("Requester", command.user.tag(), true)
                  ])
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
          })
      }).await
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