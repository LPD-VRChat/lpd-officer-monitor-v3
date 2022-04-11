mod business;
mod commands;
mod config;
mod global;

use crate::global::{Context, Data, Error};
use poise;
use poise::serenity_prelude as serenity;
use std::string::String;

#[macro_use]
extern crate lazy_static;

/// Show this menu
#[poise::command(prefix_command, slash_command, track_edits)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ?help command for more info on a command.
You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

async fn event_listener(
    _ctx: &serenity::Context,
    event: &serenity::Event,
    _framework: &poise::Framework<Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::Event::Ready(data_about_bot) => {
            println!("{} is connected!", data_about_bot.ready.user.name);
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    poise::Framework::build()
        .token(&config::CONFIG.token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .options(poise::FrameworkOptions {
            // configure framework here
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("?".into()),
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(900),
                )),
                ..Default::default()
            },
            commands: vec![help(), commands::rtv()],
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(ctx, event, framework, user_data))
            },
            ..Default::default()
        })
        .client_settings(|b| {
            b.intents(
                serenity::GatewayIntents::non_privileged()
                    | serenity::GatewayIntents::MESSAGE_CONTENT
                    | serenity::GatewayIntents::GUILD_MEMBERS,
            )
        })
        .run()
        .await
        .unwrap();
}
