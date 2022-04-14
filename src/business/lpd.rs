use entity::sea_orm::QueryFilter;
use poise::serenity_prelude as serenity;

use crate::config::CONFIG;
use crate::db;
use crate::global::{Error, OfficerCache};

use entity::officer;
use entity::officer::Entity as Officer;

pub fn has_lpd_role(roles: &[serenity::RoleId]) -> bool {
    let found_role = roles.iter().find(|role_id| role_id == &&CONFIG.roles.lpd);

    found_role.is_some()
}

pub async fn add_member(
    officer_cache: &OfficerCache,
    member: &Option<officer::Model>,
    user_id: &serenity::UserId,
) -> Result<(), Error> {
    // Create the new model, only erase the other fields if the member left the LPD for more than 7 days.
    use entity::sea_orm::entity::*;
    let err_msg = "A member that is already in the LPD can't be added to the LPD again!";
    let last_return_account_timestamp = chrono::Utc::now()
        .naive_utc()
        .checked_add_signed(chrono::Duration::days(-7))
        .ok_or("Date calculation in adding member failed because of overflow.")?
        .timestamp();
    let active_model = match member {
        Some(m) if last_return_account_timestamp > m.deleted_at.ok_or(err_msg)?.timestamp() => {
            let mut new_active_model: officer::ActiveModel = m.clone().into();
            new_active_model.deleted_at = Set(None);
            new_active_model
        }
        _ => officer::ActiveModel {
            id: Set(user_id.0),
            vrchat_name: Set("".to_owned()),
            vrchat_id: Set("".to_owned()),
            started_monitoring: Set(chrono::offset::Utc::now().naive_utc()),
            deleted_at: Set(None),
        },
    };

    // Add the user to the database
    let connection = db::establish_connection().await;
    let in_cache = member.is_some();
    let model = match in_cache {
        true => {
            Officer::update(active_model)
                .filter(officer::Column::Id.eq(user_id.0))
                .exec(&connection)
                .await?
        }
        false => {
            Officer::insert(active_model).exec(&connection).await?;
            // TODO: Change this into conversion when it is added in SeaORM:
            // https://github.com/SeaQL/sea-orm/issues/606
            Officer::find_by_id(user_id.0)
                .one(&connection)
                .await?
                .ok_or("Officer not in database after they were added. The cache would have gotten out of sync.")?
        }
    };

    // Add the new member to the cache
    let mut officer_cache_lock = officer_cache.write().await;
    let officer_cache = &mut *officer_cache_lock;
    officer_cache.insert(user_id.0, model);

    Ok(())
}

pub async fn remove_member(
    officer_cache: &OfficerCache,
    user_id: &serenity::UserId,
) -> Result<(), Error> {
    let deleted_at_date = chrono::Utc::now().naive_utc();

    // Get the officer selected from the cache
    let mut officer_cache_lock = officer_cache.write().await;
    let officer_cache = &mut *officer_cache_lock;
    let selected_officer = officer_cache
        .get_mut(&user_id.0)
        .ok_or("Officer removed from the cache between read and removal on member update.")?;

    // Update in the cache
    selected_officer.deleted_at = Some(deleted_at_date);

    // Create the update model
    use entity::sea_orm::entity::*;
    let active_model = officer::ActiveModel {
        id: Set(user_id.0),
        deleted_at: Set(Some(deleted_at_date)),
        ..Default::default()
    };

    // Update in the database
    let connection = db::establish_connection().await;
    Officer::update(active_model)
        .filter(officer::Column::Id.eq(user_id.0))
        .exec(&connection)
        .await?;

    Ok(())
}

pub async fn get_member_from_cache(
    officer_cache: &OfficerCache,
    user_id: &serenity::UserId,
) -> Option<officer::Model> {
    let officer_cache_lock = officer_cache.read().await;
    let officer_cache_map = &*officer_cache_lock;

    officer_cache_map.get(&user_id.0).cloned()
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
    is_in_cache_and(officer_cache, user_id, |model| model.deleted_at == None).await
}
