mod business;
mod commands;
mod config;
mod db;
mod global;

use crate::global::{Context, Data, Error};
use poise;
use poise::serenity_prelude as serenity;

use std::boxed::Box;
use std::collections::HashMap;
use std::iter::{Map, Zip};
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::RwLock;

use entity::sea_orm::ColumnTrait;
use entity::sea_orm::EntityTrait;
use entity::sea_orm::QueryFilter;
use tracing_subscriber;

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
        .map_err(|err| Box::from(err))
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
            let is_in_bot = business::is_in_bot(&user_data.officer_cache, &data.user.id).await;
            if !is_in_bot && business::has_lpd_role(&data.roles) {
                let mut officer_cache_lock = user_data.officer_cache.write().await;
                let officer_cache = &mut *officer_cache_lock;

                // Add the new member to the database
                let model = officer::Model {
                    id: data.user.id.0,
                    vrchat_name: "".to_owned(),
                    vrchat_id: "".to_owned(),
                    started_monitoring: chrono::offset::Utc::now().naive_utc(),
                    delete_at: None,
                };
                let active_model = officer::ActiveModel::from(model.clone());
                // println!("Active model: {:?}", active_model);

                let connection = db::establish_connection().await;
                Officer::insert(active_model).exec(&connection).await?;

                // Add the new member to the cache
                officer_cache.insert(data.user.id.0, model);
            }
        }
        // serenity::Event::GuildMemberRemove(data) => data.member.user.id,
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let connection = db::establish_connection().await;
    let officer_list = Officer::find()
        .all(&connection)
        .await
        .expect("Couldn't fetch the officers from the database.");
    let officer_ids = officer_list.iter().map(|m| m.id).collect::<Vec<_>>();
    let officer_data = HashMap::from_iter(officer_ids.into_iter().zip(officer_list));
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
