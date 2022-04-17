use entity::saved_voice_channel;
use entity::sea_orm::ColumnTrait;
use entity::sea_orm::EntityTrait;
use entity::sea_orm::QueryFilter;
use migration::DbErr;

use crate::db;
use crate::global::{Data, Error};
use poise::serenity_prelude as serenity;

/// Get a saved voice channel or create one in the database if it doesn't exist.
pub async fn get_saved_voice_channel(
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) -> Result<saved_voice_channel::Model, Error> {
    let conn = db::establish_connection().await;

    // Try to get the channel
    let try_get_channel_query = saved_voice_channel::Entity::find()
        .filter(saved_voice_channel::Column::GuildId.eq(guild_id.0))
        .filter(saved_voice_channel::Column::ChannelId.eq(guild_id.0));
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
                        .ok_or_else(|| format!("Failed to insert into the database: \"{}\"\nThe saved voice channel still couldn't be found.", err).into())
                    ,
                    _ => Err(err.into())
                }
            }
        }
    }
}

pub async fn event_listener(
    _ctx: &serenity::Context,
    event: &serenity::Event,
    _framework: &poise::Framework<Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::Event::VoiceStateUpdate(_data) => {
            println!("VoiceState Update Received");
        }
        _ => {}
    }

    Ok(())
}
