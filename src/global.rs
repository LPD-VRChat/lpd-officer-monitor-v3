use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type OfficerCache = Arc<RwLock<HashMap<u64, entity::officer::Model>>>;

pub struct Data {
    pub officer_cache: OfficerCache,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
