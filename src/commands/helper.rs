use crate::global::{Context, Error};
pub async fn send_long(ctx: Context<'_>, message: &str) -> Result<(), Error> {
    // TODO: Actually allow messages over 2000 characters
    ctx.say(message).await?;
    Ok(())
}
