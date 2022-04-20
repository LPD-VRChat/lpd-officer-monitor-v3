use poise::serenity_prelude as serenity;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[cfg(not(test))]
lazy_static! {
    pub static ref CONFIG: Config = get_config("settings.toml");
}
#[cfg(test)]
lazy_static! {
    pub static ref CONFIG: Config = get_config("test_settings.toml");
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoleConfig {
    pub lpd: serenity::RoleId,
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

pub fn get_config(file: &str) -> Config {
    // Open the file
    let config_file_path = Path::new(file);
    let display = config_file_path.display();
    let mut file = match File::open(&config_file_path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Get the file content
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read settings file.");
    // Parse the file as a toml file.
    toml::from_str(&contents).expect("Error while parsing settings file.")
}
