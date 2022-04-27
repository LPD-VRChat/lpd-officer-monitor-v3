mod business;
mod commands;
mod config;
mod db;
mod global;

use crate::global::{Context, Data, Error};
use poise::serenity_prelude as serenity;

use std::boxed::Box;
use std::string::String;
use std::vec::Vec;

// use entity::sea_orm::ColumnTrait;
// use entity::sea_orm::EntityTrait;
// use entity::sea_orm::QueryFilter;
// use tracing_subscriber;

pub use entity::officer;
pub use entity::officer::Entity as Officer;

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
    poise::builtins::help(ctx, command.as_deref(), config).await.map_err(Box::from)
}

async fn event_listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    framework: &poise::Framework<Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    business::member_management::event_listener(ctx, event, framework, user_data).await?;
    business::patrol_measure::event_listener(ctx, event, framework, user_data).await?;

    if let poise::Event::Ready { data_about_bot } = event {
        println!("{} is connected!", data_about_bot.user.name);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Setup logging
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .pretty()
    //     .with_test_writer()
    //     .init();

    poise::Framework::build()
        .token(&config::CONFIG.token)
        .user_data_setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                // Initialize the slash commands
                poise::serenity_prelude::GuildId(config::CONFIG.guild_id)
                    .set_application_commands(ctx, |b| {
                        *b = poise::samples::create_application_commands(
                            &framework.options().commands,
                        );
                        b
                    })
                    .await
                    .unwrap();

                // Ready the user data
                Ok(Data {
                    officer_cache: business::member_management::cache_init().await,
                    patrol_cache: business::patrol_measure::cache_init().await,
                })
            })
        })
        .options(poise::FrameworkOptions {
            // configure framework here
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("?".into()),
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(900),
                )),
                ..Default::default()
            },
            commands: vec![help(), commands::patrol_time(), commands::rtv()],
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(ctx, event, framework, user_data))
            },
            ..Default::default()
        })
        .intents(
            serenity::GatewayIntents::non_privileged()
                | serenity::GatewayIntents::MESSAGE_CONTENT
                | serenity::GatewayIntents::GUILD_MEMBERS
                | serenity::GatewayIntents::GUILD_PRESENCES,
        )
        .run()
        .await
        .unwrap();
}
