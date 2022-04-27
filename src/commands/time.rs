use super::helper::send_long;
use crate::business as bs;
use crate::global::{Context, Error};
use poise::serenity_prelude as serenity;

fn date_from_days(days: i64) -> Result<chrono::NaiveDate, Error> {
    Ok(chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(days))
        .ok_or("From date overflow!")?
        .naive_utc()
        .date())
}

/// Convert seconds into weeks, days, hours, minutes and seconds
fn split_duration(seconds: i64) -> (i64, i64, i64, i64, i64) {
    // Calculate each duration
    let (min_rem, seconds) = (seconds / 60, seconds % 60);
    let (hour_rem, minutes) = (min_rem / 60, min_rem % 60);
    let (day_rem, hours) = (hour_rem / 60, hour_rem % 60);
    let (weeks, days) = (day_rem / 60, day_rem % 60);

    // Return the values
    (weeks, days, hours, minutes, seconds)
}

/// Make a multi line string that represents a duration
#[rustfmt::skip]
fn display_duration_multiline(seconds: i64) -> String {
    let (weeks, days, hours, minutes, seconds) = split_duration(seconds);

    // Convert the duration into a string
    let mut return_str = "".to_owned();
    if return_str.len() != 0 || weeks != 0 { return_str += &format!("Weeks: {}\n", weeks) }
    if return_str.len() != 0 || days != 0 { return_str += &format!("Days: {}\n", days) }
    if return_str.len() != 0 || hours != 0 { return_str += &format!("Hours: {}\n", hours) }
    if return_str.len() != 0 || minutes != 0 { return_str += &format!("Minutes: {}\n", minutes) }
    return_str + &format!("Seconds: {}", seconds)
}

/// Make a single line string that represents a duration
fn display_duration(seconds: i64) -> String {
    let (weeks, days, hours, minutes, seconds) = split_duration(seconds);

    // Convert the duration into a single line string
    format!("{}:{}:{}:{}:{}", weeks, days, hours, minutes, seconds)
}

/// Check patrol time of an officer.
#[poise::command(prefix_command, slash_command, track_edits, category = "Time")]
pub async fn patrol_time(
    ctx: Context<'_>,
    #[description = "The number of days to look back for activity, this defaults to 28."]
    days: Option<i64>,
    #[description = "From date in the format YYYY-MM-DD. This default to the same value as days."]
    from_date: Option<chrono::NaiveDate>,
    #[description = "To date in the format YYYY-MM-DD. This is set to the current date if it isn't given."]
    to_date: Option<chrono::NaiveDate>,
    #[description = "List all the patrols in the time period specified, defaults to false."]
    list_patrols: Option<bool>,
    #[description = "The officer to get the patrol time from."] officer: serenity::User,
) -> Result<(), Error> {
    // Setup the parameters
    let list_patrols = list_patrols.unwrap_or(false);
    let to_date = to_date.unwrap_or_else(|| chrono::Utc::now().naive_utc().date());
    let from_date = match (days, from_date) {
        (Some(_), Some(_)) => {
            return Err("days and from_date can't both be provided at the same time.".into());
        }
        (None, Some(from_date)) => from_date,
        (Some(days), None) => date_from_days(days)?,
        (None, None) => date_from_days(28)?,
    };

    let time_str = match list_patrols {
        true => {
            let patrols = bs::patrol_measure::get_patrols(
                from_date.and_hms(0, 0, 0),
                to_date.and_hms(23, 59, 59),
                officer.id,
            )
            .await?;
            let result = patrols.into_iter().fold(String::new(), |acc, item| {
                // Get the duration for this patrol
                let patrol_duration =
                    display_duration(item.0.end.signed_duration_since(item.0.start).num_seconds());

                // Combine the patrol_voice objects
                let patrol_voices = item.1.into_iter().fold(String::new(), |acc, pat_vc| {
                    let pat_dur_sec = pat_vc.end.signed_duration_since(pat_vc.start).num_seconds();
                    let pat_vc_dur = display_duration(pat_dur_sec);
                    format!("{}    {} - {}\n", acc, pat_vc.start.to_string(), pat_vc_dur)
                });

                // Combine the data for this patrol, including the patrol_voice objects
                format!(
                    "{}{} - {}\n{}\n",
                    acc,
                    item.0.start.to_string(),
                    patrol_duration.to_string(),
                    &patrol_voices[0..patrol_voices.len().checked_sub(1).unwrap_or(0)]
                )
            });
            let cutoff_result = &result[0..result.len().checked_sub(1).unwrap_or(0)];
            format!("```\n{}```", cutoff_result)
        }
        false => {
            let patrol_time = bs::patrol_measure::get_patrol_time(
                from_date.and_hms(0, 0, 0),
                to_date.and_hms(23, 59, 59),
                officer.id,
            )
            .await?;
            display_duration_multiline(patrol_time)
        }
    };

    let message =
        format!("On duty time for {} - from {} to {}:\n{}", officer, from_date, to_date, time_str);
    send_long(ctx, &message).await?;

    Ok(())
}
