mod business;
mod commands;
mod config;
mod db;
mod global;

use crate::global::{Context, Data, Error};
use poise::serenity_prelude as serenity;

use std::boxed::Box;
use std::collections::HashMap;
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::RwLock;

// use entity::sea_orm::ColumnTrait;
use entity::sea_orm::EntityTrait;
use entity::sea_orm::QueryFilter;
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
    poise::builtins::help(ctx, command.as_deref(), config)
        .await
        .map_err(Box::from)
}

async fn event_listener(
    _ctx: &serenity::Context,
    event: &serenity::Event,
    _framework: &poise::Framework<Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::Event::Ready(data_about_bot) => {
            println!("{} is connected!", data_about_bot.ready.user.name);
        }
        serenity::Event::GuildMemberUpdate(data) => {
            let is_in_bot_cache =
                business::is_lpd_in_cache(&user_data.officer_cache, &data.user.id).await;

            // Add the user to the database if they just got an LPD role
            if !is_in_bot_cache && business::has_lpd_role(&data.roles) {
                // Create the new model
                use entity::sea_orm::entity::*;
                let active_model = officer::ActiveModel {
                    id: Set(data.user.id.0),
                    vrchat_name: Set("".to_owned()),
                    vrchat_id: Set("".to_owned()),
                    started_monitoring: Set(chrono::offset::Utc::now().naive_utc()),
                    deleted_at: Set(None),
                };

                // Add the user to the database
                let connection = db::establish_connection().await;
                let in_cache = business::is_in_cache(&user_data.officer_cache, &data.user.id).await;
                let model = match in_cache {
                    true => {
                        Officer::update(active_model)
                            .filter(officer::Column::Id.eq(data.user.id.0))
                            .exec(&connection)
                            .await?
                    }
                    false => {
                        Officer::insert(active_model).exec(&connection).await?;
                        Officer::find_by_id(data.user.id.0)
                            .one(&connection)
                            .await?
                            .expect("Officer not in database after they were added.")
                    }
                };

                // Add the new member to the cache
                let mut officer_cache_lock = user_data.officer_cache.write().await;
                let officer_cache = &mut *officer_cache_lock;
                officer_cache.insert(data.user.id.0, model);
            }
            // Remove an officer if they no longer have the LPD roles
            else if is_in_bot_cache && !business::has_lpd_role(&data.roles) {
                let deleted_at_date = chrono::Utc::now().naive_utc();

                // Get the officer selected from the cache
                let mut officer_cache_lock = user_data.officer_cache.write().await;
                let officer_cache = &mut *officer_cache_lock;
                let selected_officer = officer_cache.get_mut(&data.user.id.0).expect(
                    "Officer removed from the cache between read and removal on member update.",
                );

                // Update in the cache
                selected_officer.deleted_at = Some(deleted_at_date);

                // Create the update model
                use entity::sea_orm::entity::*;
                let active_model = officer::ActiveModel {
                    id: Set(data.user.id.0),
                    deleted_at: Set(Some(deleted_at_date)),
                    ..Default::default()
                };

                // Update in the database
                let connection = db::establish_connection().await;
                Officer::update(active_model)
                    .filter(officer::Column::Id.eq(data.user.id.0))
                    .exec(&connection)
                    .await?;
            };
        }
        // serenity::Event::GuildMemberRemove(data) => data.member.user.id,
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Fill in the officer cache with all the officers from the database
    let connection = db::establish_connection().await;
    let officer_list = Officer::find()
        .all(&connection)
        .await
        .expect("Couldn't fetch the officers from the database.");
    let officer_data: HashMap<_, _> = officer_list.into_iter().map(|m| (m.id, m)).collect();
    let officer_cache = Arc::new(RwLock::new(officer_data));

    let ctx_data = Data {
        officer_cache: officer_cache.clone(),
    };

    // Setup logging
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .pretty()
    //     .with_test_writer()
    //     .init();

    poise::Framework::build()
        .token(&config::CONFIG.token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(ctx_data) }))
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
