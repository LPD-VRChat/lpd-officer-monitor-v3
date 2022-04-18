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
pub async fn is_on_patrol(
    patrol_cache: &PatrolCache,
    user_id: &serenity::UserId,
) -> Result<bool, Error> {
    // Get a read lock to the patrol cache
    let patrol_cache_lock = patrol_cache.read().await;
    let patrol_cache_map = &*patrol_cache_lock;

    let err_msg = format!("There was an officer in the cache ({}) with no channel_logs, this shouldn't be possible as the minimum is always one.", user_id);
    match patrol_cache_map.get(&user_id.0) {
        Some(patrol_log) => Ok(patrol_log
            .voice_log
            .last()
            .ok_or::<Error>(err_msg.into())?
            .end
            .is_none()),
        None => Ok(false),
    }
}

/// Register a user going on duty
async fn go_on_duty(
    patrol_cache: &PatrolCache,
    user_id: serenity::UserId,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) -> Result<(), Error> {
    // Make sure we don't keep the lock longer than we need to
    let return_value = {
        // Get a write lock to the cache
        let mut patrol_cache_lock = patrol_cache.write().await;
        let patrol_cache_map = &mut *patrol_cache_lock;

        // Add the patrol to the cache
        patrol_cache_map.insert(
            user_id.0,
            PatrolLog {
                officer_id: user_id,
                voice_log: vec![ChannelLog {
                    guild_id,
                    channel_id,
                    start: chrono::Utc::now().naive_utc(),
                    end: None,
                }],
            },
        )
    };

    // Throw an error if the user already existed in the cache
    match return_value {
        Some(dropped_model) => Err(format!(
            "PatrolLog dropped as someone was already on duty when go_on_duty was called on them!\nOfficer: {}\nDropped log: {:?}",
            user_id, dropped_model
        )
        .into()),
        None => Ok(()),
    }
}

/// Register a user going off duty
async fn go_off_duty(patrol_cache: &PatrolCache, user_id: serenity::UserId) {}
/// Register a user switching on duty comms
async fn move_on_duty_vc(
    patrol_cache: &PatrolCache,
    user_id: serenity::UserId,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) {
}

/// Check if a channel is being ignored according to the bots settings
fn is_ignored_channel(channel_id: serenity::ChannelId) -> bool {
    // TODO: Add a way to ignore specific channels
    false
}

/// Check if a channel is being monitored according to the bots settings
fn is_monitored(channel_id: serenity::ChannelId, category_id: Option<serenity::ChannelId>) -> bool {
    // Check if the category exists and is monitored
    if let Some(category_id) = category_id {
        if CONFIG
            .patrol_time
            .monitored_categories
            .contains(&category_id.0)
        {
            return !is_ignored_channel(channel_id);
        }
    }

    // If that hasn't returned true, check if the channel is monitored
    if CONFIG
        .patrol_time
        .monitored_channels
        .contains(&channel_id.0)
    {
        return !is_ignored_channel(channel_id);
    }

    // Neither the category ir channel were monitored, the channel then can't be monitored
    false
}

pub async fn event_listener(
    ctx: &serenity::Context,
    event: &serenity::Event,
    _framework: &poise::Framework<Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::Event::Ready(_data) => {
            // TODO: Add people that are on patrol when the bot starts
            println!("Patrol Measurement Ready!")
        }
        serenity::Event::VoiceStateUpdate(data) => match data.voice_state.guild_id {
            // Measure patrol time in the main LPD server
            Some(guild_id) if guild_id.0 == CONFIG.guild_id => {
                // Ready variables to simplify the code
                let user_id = data.voice_state.user_id;
                let patrol_cache = &user_data.patrol_cache;
                let on_patrol = is_on_patrol(patrol_cache, &user_id).await?;
                let get_category_id = |c| ctx.cache.channel_category_id(c);

                match data.voice_state.channel_id {
                    // Someone is going on duty or switching on duty comms
                    Some(channel_id) if is_monitored(channel_id, get_category_id(channel_id)) => {
                        match on_patrol {
                            true => {
                                // Someone is moving from voice channel to the other
                                move_on_duty_vc(patrol_cache, user_id, guild_id, channel_id).await;
                            }
                            false => {
                                // Someone is going on duty
                                go_on_duty(patrol_cache, user_id, guild_id, channel_id).await;
                            }
                        }
                    }
                    // Someone is leaving on duty comms
                    None if on_patrol => {
                        // Someone is going off duty
                        go_off_duty(patrol_cache, user_id).await;
                    }
                    _ => {}
                }
            }
            _ => (),
        },
        _ => {}
    }

    Ok(())
}
