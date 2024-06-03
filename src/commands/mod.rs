use crate::config::ConfigStorage;
use crate::constants::EMBED_COLOUR;
use serenity::builder::CreateEmbed;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::builder::EditInteractionResponse;
use serenity::model::application::CommandInteraction;
use serenity::model::prelude::Ready;
use serenity::prelude::Context;
use serenity::Error;
use serenity::{async_trait, builder::CreateCommand};
use tracing::{error, info};

mod cmd;
mod playback;
mod utils;

static COMMAND_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(10);

#[async_trait]
trait Command {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error>;
  fn info() -> CreateCommand;
  fn name() -> &'static str;
}

pub async fn register_commands(ctx: &Context, _ready: &Ready) {
  let config_lock = {
    let data = ctx.data.read().await;
    data
      .get::<ConfigStorage>()
      .expect("No config in global storage")
      .clone()
  };

  if let Some(guild) = config_lock.guild_id {
    let commands = guild.set_commands(&ctx.http, command_list()).await;

    match commands {
      Ok(c) => {
        let cmd_list = c.iter().fold("".to_string(), |mut a, c| {
          let s = format!("{}\n", c.name);
          a.push_str(&s);
          a
        });
        info!("Added commands for Guild({}):\n{}", guild, cmd_list)
      }
      Err(e) => panic!("Couldn't set application commands: {:#?}", e),
    }
  } else {
    let commands = ctx.http.create_global_commands(&command_list()).await;

    match commands {
      Ok(c) => {
        let cmd_list = c.iter().fold("".to_string(), |mut a, c| {
          let s = format!("{}\n", c.name);
          a.push_str(&s);
          a
        });
        info!("Added global commands:\n{}", cmd_list)
      }
      Err(e) => panic!("Couldn't set global application commands: {:#?}", e),
    }
  }
}

fn command_list() -> Vec<CreateCommand> {
  vec![
    cmd::Join::info(),
    cmd::Leave::info(),
    cmd::Play::info(),
    cmd::Capybara::info(),
    cmd::Seek::info(),
    cmd::Skip::info(),
    cmd::Queue::info(),
    cmd::Me::info(),
    cmd::Info::info(),
    cmd::Stop::info(),
    cmd::Eval::info(),
    cmd::Pause::info(),
    cmd::Resume::info(),
  ]
}

pub async fn handle_commands(ctx: &Context, command: CommandInteraction) {
  let name = command.data.name.clone();
  let user = command.user.clone();
  match command
    .create_response(
      &ctx.http,
      serenity::builder::CreateInteractionResponse::Defer(
        CreateInteractionResponseMessage::new().content("Loading"),
      ),
    )
    .await
  {
    Ok(_) => info!("{} command deferred", name),
    Err(e) => error!("Error deferring command {}: {}", name, e),
  }

  let result = match name.as_str() {
    _ if name == cmd::Join::name() => cmd::Join::execute(ctx, &command),
    _ if name == cmd::Leave::name() => cmd::Leave::execute(ctx, &command),
    _ if name == cmd::Play::name() => cmd::Play::execute(ctx, &command),
    _ if name == cmd::Seek::name() => cmd::Seek::execute(ctx, &command),
    _ if name == cmd::Skip::name() => cmd::Skip::execute(ctx, &command),
    _ if name == cmd::Queue::name() => cmd::Queue::execute(ctx, &command),
    _ if name == cmd::Stop::name() => cmd::Stop::execute(ctx, &command),
    _ if name == cmd::Capybara::name() => cmd::Capybara::execute(ctx, &command),
    _ if name == cmd::Me::name() => cmd::Me::execute(ctx, &command),
    _ if name == cmd::Info::name() => cmd::Info::execute(ctx, &command),
    _ if name == cmd::Eval::name() => cmd::Eval::execute(ctx, &command),
    _ if name == cmd::Pause::name() => cmd::Pause::execute(ctx, &command),
    _ if name == cmd::Resume::name() => cmd::Resume::execute(ctx, &command),
    _ => Box::pin(text_response(ctx, &command, "Invalid command")),
  };

  match tokio::time::timeout(COMMAND_TIMEOUT, result).await {
    Ok(result) => {
      if let Err(e) = result {
        error!("Couldn't respond to command: {}", e);
        error!(
          "{user} failed running command {cmd}",
          user = user.tag(),
          cmd = name
        );
        text_response(ctx, &command, "Error processing command")
          .await
          .unwrap_or(());
      } else {
        info!("{user} ran command {cmd}", user = user.tag(), cmd = name)
      }
    }
    Err(e) => {
      error!("Couldn't respond to command: {}", e);
      error!(
        "{user} failed running command {cmd}",
        user = user.tag(),
        cmd = name
      );
      text_response(ctx, &command, "Took too long processing command")
        .await
        .unwrap_or(());
    }
  }
}

pub async fn text_response<D>(
  ctx: &Context,
  command: &CommandInteraction,
  text: D,
) -> Result<(), Error>
where
  std::string::String: From<D>,
{
  match command
    .edit_response(
      &ctx.http,
      EditInteractionResponse::new().embed(CreateEmbed::new().title(text).colour(EMBED_COLOUR)),
    )
    .await
  {
    Ok(_) => Ok(()),
    Err(e) => Err(e),
  }
}
