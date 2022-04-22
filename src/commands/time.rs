use crate::business as bs;
use crate::global::{Context, Error};
use poise::serenity_prelude as serenity;

/// Check patrol time of an officer
#[poise::command(prefix_command, slash_command, track_edits, category = "Time")]
pub async fn patrol_time(
    ctx: Context<'_>,
    #[description = "From date"] from_date: Option<chrono::NaiveDate>,
    #[description = "To date"] to_date: Option<chrono::NaiveDate>,
    #[description = "Officer to get the time of"] officer: serenity::User,
) -> Result<(), Error> {
    ctx.say(from_date.unwrap().to_string()).await?;
    ctx.say(officer.name).await?;
    Ok(())
}
