use entity::sea_orm::EntityTrait;
use entity::sea_orm::QueryFilter;
use poise::serenity_prelude as serenity;

use crate::config::CONFIG;
use crate::db;
use crate::global::{Data, Error, OfficerCache};

use entity::officer;
use entity::officer::Entity as Officer;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn has_lpd_role(roles: &[serenity::RoleId]) -> bool {
    // TODO: Add the other LPD roles.
    let found_role = roles.iter().find(|role_id| role_id == &&CONFIG.roles.lpd);

    found_role.is_some()
}

pub async fn get_member_from_cache(
    officer_cache: &OfficerCache,
    user_id: &serenity::UserId,
) -> Option<officer::Model> {
    let officer_cache_lock = officer_cache.read().await;
    let officer_cache_map = &*officer_cache_lock;

    officer_cache_map.get(&user_id.0).cloned()
}

// pub async fn is_in_cache_and<F>(
//     officer_cache: &OfficerCache,
//     user_id: &serenity::UserId,
//     and_fn: F,
// ) -> bool
// where
//     F: Send + Sync + Fn(&officer::Model) -> bool,
// {
//     let officer_cache_lock = officer_cache.read().await;
//     let officer_cache_map = &*officer_cache_lock;
//     match officer_cache_map.get(&user_id.0) {
//         Some(val) => and_fn(val),
//         None => false,
//     }
// }

// pub async fn is_in_cache(officer_cache: &OfficerCache, user_id: &serenity::UserId) -> bool {
//     is_in_cache_and(officer_cache, user_id, |_m| true).await
// }

// pub async fn is_lpd_in_cache(officer_cache: &OfficerCache, user_id: &serenity::UserId) -> bool {
//     is_in_cache_and(officer_cache, user_id, |model| model.deleted_at == None).await
// }

async fn add_member(
    officer_cache: &OfficerCache,
    member: &Option<officer::Model>,
    user_id: &serenity::UserId,
) -> Result<(), Error> {
    // Create the new model, only erase the other fields if the member left the LPD for more than 7 days.
    use entity::sea_orm::entity::*;
    let err_msg = "A member that is already in the LPD can't be added to the LPD again!";
    let last_allowed_return = chrono::Utc::now()
        .naive_utc()
        .checked_add_signed(chrono::Duration::days(-7))
        .ok_or("Date calculation in adding member failed because of overflow.")?;
    let active_model = match member {
        Some(m) if m.deleted_at.ok_or(err_msg)? > last_allowed_return => {
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

async fn remove_member(
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

pub async fn event_listener(
    _ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: &poise::Framework<Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::GuildMemberUpdate { old_if_available: _, new } => {
            let member = get_member_from_cache(&user_data.officer_cache, &new.user.id).await;
            let in_cache_and_lpd = match member {
                Some(ref m) => m.deleted_at.is_none(),
                None => false,
            };

            // Add the user to the database if they just got an LPD role but aren't in the cache yet
            // TODO: Change add_member and remove_member into transactions to allow for better error
            // handling mid way through.
            if !in_cache_and_lpd && has_lpd_role(&new.roles) {
                add_member(&user_data.officer_cache, &member, &new.user.id)
                    .await
                    .expect("Failed adding member on role change.");
                println!(
                    "Added member {} ({}) ({}) as they just got the LPD role.",
                    &new.user, &new.user.name, &new.user.id
                );
            }
            // Remove an officer if they no longer have the LPD roles
            else if in_cache_and_lpd && !has_lpd_role(&new.roles) {
                remove_member(&user_data.officer_cache, &new.user.id)
                    .await
                    .expect("Failed removing member on role change.");
                println!(
                    "Removed member {} ({}) ({}) as they no longer have the LPD role.",
                    &new.user, &new.user.name, &new.user.id
                );
            };
        }
        poise::Event::GuildMemberRemoval { guild_id: _, user, member_data_if_available: _ } => {
            remove_member(&user_data.officer_cache, &user.id)
                .await
                .expect("Failed removing member on server leave.");
            println!(
                "Removed member {} ({}) ({}) as they no longer have the LPD role.",
                &user, &user.name, &user.id
            );
        }
        _ => {}
    }

    Ok(())
}

pub async fn cache_init() -> OfficerCache {
    // Fill in the officer cache with all the officers from the database
    let connection = db::establish_connection().await;
    let officer_list = Officer::find()
        .all(&connection)
        .await
        .expect("Couldn't fetch the officers from the database.");
    let officer_data: HashMap<_, _> = officer_list.into_iter().map(|m| (m.id, m)).collect();
    Arc::new(RwLock::new(officer_data))
}
