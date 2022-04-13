use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Data {
    pub officer_cache: Arc<RwLock<HashMap<u64, entity::officer::Model>>>,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
