use crate::business as bs;
use crate::global::{Context, Error};

/// Display your or another user's account creation date
#[poise::command(prefix_command, slash_command, track_edits)]
pub async fn rtv(
    ctx: Context<'_>,
    #[description = "Role name"] role_name: String,
) -> Result<(), Error> {
    // Get the role by its name
    let role = match bs::get_role_by_decorated_name(&ctx.discord().cache, &role_name).await {
        Some(role) => role,
        None => {
            ctx.say(format!("Couldn't find role `{}`", role_name)).await?;
            return Ok(());
        }
    };

    // Get the members and format them into something printable.
    let output = bs::get_role_members(ctx.discord(), &role.id)
        .await
        .into_iter()
        // Get the server nickname or username if they don't have a nickname
        .map(|m| m.nick.unwrap_or(m.user.name))
        // Join into string separated with \n
        .collect::<Vec<_>>()
        .join("\n");

    ctx.say(format!("Everyone in the role `{role_name}`:\n```\n{output}\n```")).await?;

    Ok(())
}
