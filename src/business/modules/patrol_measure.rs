use entity::saved_voice_channel;
use entity::sea_orm::ColumnTrait;
use entity::sea_orm::EntityTrait;
use entity::sea_orm::QueryFilter;

use crate::config::CONFIG;
use crate::db;
use crate::global::{Data, Error, PatrolCache};
use migration::DbErr;
use poise::serenity_prelude as serenity;

macro_rules! some_or_return {
    ($x:expr, $y:expr) => {
        if let Some(x) = $x {
            x
        } else {
            return $y;
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct ChannelLog {
    pub guild_id: serenity::GuildId,
    pub channel_id: serenity::ChannelId,
    pub start: chrono::NaiveDateTime,
    pub end: Option<chrono::NaiveDateTime>,
}
#[derive(Debug, Clone)]
pub struct PatrolLog {
    pub officer_id: serenity::UserId,
    pub voice_log: Vec<ChannelLog>,
}

/// Get a saved voice channel or create one in the database if it doesn't exist.
pub async fn get_saved_voice_channel(
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) -> Result<saved_voice_channel::Model, Error> {
    let conn = db::establish_connection().await;

    // Try to get the channel
    let try_get_channel_query = saved_voice_channel::Entity::find()
        .filter(saved_voice_channel::Column::GuildId.eq(guild_id.0))
        .filter(saved_voice_channel::Column::ChannelId.eq(channel_id.0));
    let channel = try_get_channel_query.clone().one(&conn).await?;

    match channel {
        // Channel can just be returned as it existed already
        Some(c) => Ok(c),
        None => {
            // The channel doesn't exist, create it instead
            use entity::sea_orm::entity::*;
            let active_model = saved_voice_channel::ActiveModel {
                guild_id: Set(guild_id.0),
                channel_id: Set(channel_id.0),
                name: Set("".to_owned()),
                ..Default::default()
            };

            // Try to get the model anyway if there is a DB error as it may just mean the channel
            // has already been added by another thread
            let save_result = active_model.save(&conn).await;
            match save_result {
                Ok(active_model) => Ok(
                    // TODO: Change this into automatic conversion once that is added to SeaORM
                    saved_voice_channel::Model {
                        id: active_model.id.as_ref().clone(),
                        guild_id: guild_id.0,
                        channel_id: channel_id.0,
                        name: "".to_owned()
                    }
                ),
                Err(err) => match err {
                    // Ignore the error if it failed inserting as it may have been because of
                    // another thread, meaning it can still be fetched
                    DbErr::Exec(_) => try_get_channel_query
                        .one(&conn)
                        .await?
                        .ok_or_else(|| format!("Failed to insert into the database: \"{}\"\nThe saved voice channel still couldn't be found.", err).into()),
                    _ => Err(err.into())
                }
            }
        }
    }
}

/// Check if a officer is on patrol at the moment.
///
/// This function panics if the officer is in the cache but their patrol has no voice logs as there
/// should always be at a minimum 1 voice log with some start time but not necessarily an end time.
pub async fn is_on_patrol(patrol_cache: &PatrolCache, user_id: &serenity::UserId) -> bool {
    let patrol_cache_lock = patrol_cache.read().await;
    let patrol_cache_map = &*patrol_cache_lock;

    match patrol_cache_map.get(&user_id.0) {
        Some(patrol_log) => patrol_log.voice_log.last().unwrap().end.is_none(),
        None => false,
    }
}

pub async fn event_listener(
    ctx: &serenity::Context,
    event: &serenity::Event,
    _framework: &poise::Framework<Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::Event::Ready(_data) => println!("Patrol Measurement Ready!"),
        serenity::Event::VoiceStateUpdate(data) => match data.voice_state.guild_id {
            // Measure patrol time in the main LPD server
            Some(guild_id) if guild_id.0 == CONFIG.guild_id => {
                let on_patrol =
                    is_on_patrol(&user_data.patrol_cache, &data.voice_state.user_id).await;

                match data.voice_state.channel_id {
                    Some(channel_id) => {
                        // Someone may be going on duty
                        if channel_id
                    }
                    None => if on_patrol {
                        // Someone may be going off duty
                    }
                    _ => {}
                }

                // Check if that channel is monitored
                if CONFIG
                    .patrol_time
                    .monitored_channels
                    .contains(&channel_id.0)
                {}

                // Make sure the channel is in a monitored category
                let category_id =
                    some_or_return!(ctx.cache.channel_category_id(channel_id), Ok(()));
                if CONFIG
                    .patrol_time
                    .monitored_categories
                    .contains(&category_id.0)
                {
                    // Measure the time from/to the channel
                }
            }
            _ => (),
        },
        _ => {}
    }

    Ok(())
}
