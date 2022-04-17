use crate::business::patrol_measure;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type OfficerCache = Arc<RwLock<HashMap<u64, entity::officer::Model>>>;
pub type PatrolCache = Arc<RwLock<HashMap<u64, patrol_measure::PatrolLog>>>;

pub struct Data {
    pub officer_cache: OfficerCache,
    pub patrol_cache: PatrolCache,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
