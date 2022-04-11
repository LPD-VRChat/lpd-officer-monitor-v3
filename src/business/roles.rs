use crate::config::CONFIG;
use ::serenity::futures::StreamExt;
use poise::serenity_prelude as serenity;
use std::char;
use std::future;
use std::panic;
use std::str::FromStr;
use std::string::String;
use std::sync::Arc;

pub fn remove_role_decoration(role_name: &str) -> String {
    let chars_to_remove = [' ', '|', '\u{2800}', ' '];
    let should_be_trimmed = |chr: char| chars_to_remove.contains(&chr);
    let trimmed_name = role_name.trim_matches(should_be_trimmed);
    String::from_str(trimmed_name).expect("Infallible")
}

pub async fn get_role_by_decorated_name(
    cache: &Arc<serenity::Cache>,
    role_name: &str,
) -> Option<serenity::Role> {
    // Find the role
    let role = cache
        .guild_roles(CONFIG.guild_id)
        .expect(&CONFIG.guild_error_text)
        .into_values()
        .find(|x| remove_role_decoration(&x.name) == role_name);

    // Clone the role if it was found
    match role {
        Some(res) => Some(res.clone()),
        None => None,
    }
}

pub async fn get_role_members(
    ctx: &poise::serenity_prelude::Context,
    role_id: &serenity::RoleId,
) -> Vec<serenity::Member> {
    serenity::GuildId(CONFIG.guild_id)
        // Get an iterator over all of the members
        .members_iter(&ctx)
        // Filter out anyone that isn't in the role or returns an error.
        .filter(|m| {
            let has_role = match m {
                Ok(member) => member.roles.contains(&role_id),
                Err(err) => {
                    println!(
                        "get_role_members: members_iter.filter got an error: {:?}",
                        err
                    );
                    false
                }
            };

            future::ready(has_role)
        })
        // Remove the Result and turn it into Members.
        .map(|m| match m {
            Ok(member) => member,
            // This shouldn't be possible as its filtered out in the filter above.
            Err(err) => panic!("Impossible state: {:?}", err),
        })
        // Turn the Future iterator into a vector.
        .collect::<Vec<_>>()
        .await
}
