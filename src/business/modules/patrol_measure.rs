// use super::member_management;
use crate::global::{Data, Error};
use poise::serenity_prelude as serenity;

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
