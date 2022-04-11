use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

lazy_static! {
    pub static ref CONFIG: Config = get_config("settings.toml");
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub token: String,
    pub guild_id: u64,
    pub guild_error_text: String,
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
