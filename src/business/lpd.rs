use poise::serenity_prelude as serenity;

use crate::config::CONFIG;
use crate::global::OfficerCache;
use entity::officer;

pub fn has_lpd_role(roles: &[serenity::RoleId]) -> bool {
    let found_role = roles.iter().find(|role_id| role_id == &&CONFIG.roles.lpd);

    found_role.is_some()
}

pub async fn is_in_cache_and<F>(
    officer_cache: &OfficerCache,
    user_id: &serenity::UserId,
    and_fn: F,
) -> bool
where
    F: Send + Sync + Fn(&officer::Model) -> bool,
{
    let officer_cache_lock = officer_cache.read().await;
    let officer_cache_map = &*officer_cache_lock;

    match officer_cache_map.get(&user_id.0) {
        Some(val) => and_fn(val),
        None => false,
    }
}

pub async fn is_in_cache(officer_cache: &OfficerCache, user_id: &serenity::UserId) -> bool {
    is_in_cache_and(officer_cache, user_id, |_m| true).await
}

pub async fn is_lpd_in_cache(officer_cache: &OfficerCache, user_id: &serenity::UserId) -> bool {
    is_in_cache_and(officer_cache, user_id, |model| model.delete_at == None).await
}
