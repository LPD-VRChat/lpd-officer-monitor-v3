use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use poise::serenity_prelude as serenity;

use crate::config::CONFIG;
use entity::officer;

pub fn has_lpd_role(roles: &Vec<serenity::RoleId>) -> bool {
    let found_role = roles.iter().find(|role_id| role_id == &&CONFIG.roles.lpd);

    match found_role {
        Some(_) => true,
        None => false,
    }
}

pub async fn is_in_bot(
    officer_cache: &Arc<RwLock<HashMap<u64, officer::Model>>>,
    user_id: &serenity::UserId,
) -> bool {
    let officer_cache_lock = officer_cache.read().await;
    let officer_cache_map = &*officer_cache_lock;

    match officer_cache_map.get(&user_id.0) {
        Some(val) => val.delete_at == None,
        None => false,
    }
}
