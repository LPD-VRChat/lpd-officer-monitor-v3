use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use std::collections::HashSet;

#[cfg(not(test))]
lazy_static! {
    pub static ref CONFIG: Config = get_config("settings/test.toml", true);
}
#[cfg(test)]
lazy_static! {
    pub static ref CONFIG: Config = get_config("settings/unit_test.toml", false);
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoleConfig {
    pub lpd: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PatrolTime {
    pub monitored_categories: HashSet<u64>,
    pub monitored_channels: HashSet<u64>,
    pub ignored_channels: HashSet<u64>,
    pub bad_main_channel_starts: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub token: String,
    pub guild_id: u64,
    pub guild_error_text: String,
    pub roles: RoleConfig,
    pub patrol_time: PatrolTime,
}
/// Calculate the parent folder path with a slash at the end.
/// Returns an empty string if the path sent in doesn't include any folder and is just a file.
fn get_parent_folder(file: &str) -> String {
    match file.rfind("/") {
        Some(location) => file.split_at(location).0.to_owned() + "/",
        None => "".to_owned(),
    }
}

pub fn get_config(file: &str, include_local: bool) -> Config {
    let parent_path = get_parent_folder(file);

    // Get the config
    let mut figment =
        Figment::new().merge(Toml::file(parent_path.clone() + "base.toml")).merge(Toml::file(file));

    // Only include local config if it is supposed to (don't want it messing with unit testing)
    if include_local {
        figment =
            figment.merge(Toml::file(parent_path + "local.toml")).merge(Env::prefixed("LOM_"));
    }

    // Return the resulting config objects
    let config = figment.extract().expect("Failed to load config file");

    // Show the current configuration
    println!("Configuration: {:?}", config);

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_get_parent_folder() {
        assert_eq!(get_parent_folder("settings.toml"), "".to_owned());
        assert_eq!(get_parent_folder("/settings.toml"), "/".to_owned());
        assert_eq!(get_parent_folder("settings/main.toml"), "settings/".to_owned());
        assert_eq!(get_parent_folder("/settings/main.toml"), "/settings/".to_owned());
    }
}
