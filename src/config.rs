use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    hotkeys::{EventTapHandle, Shortcut, global_handler},
    subscriber::{Action, ActionSender},
};

pub fn home() -> String {
    std::env::var("HOME").unwrap()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    hotkeys: HashMap<String, Action>,
    inner_gaps: GapsConfig,
    outer_gaps: GapsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkeys: HashMap::new(),
            inner_gaps: GapsConfig::default(),
            outer_gaps: GapsConfig::default(),
        }
    }
}

impl Config {
    pub fn get_config_path() -> PathBuf {
        let env_args = std::env::args().collect::<Vec<String>>();
        let config_file_path = env_args
            .first()
            .map(String::clone)
            .unwrap_or(home() + "/.config/spacewm/config.toml");
        PathBuf::from_str(&config_file_path).unwrap()
    }
    pub fn load_config() -> Option<Config> {
        let config_string = fs::read_to_string(Config::get_config_path()).ok()?;

        toml::from_str(&config_string).ok()
    }

    pub fn create_default_config(path: PathBuf) {
        fs::write(path, toml::to_string(&Config::default()).unwrap()).unwrap_or_default()
    }

    pub fn register_hotkeys(&self, sender: ActionSender) -> EventTapHandle {
        let a = self
            .hotkeys
            .iter()
            .filter_map(|(hotkey, a)| {
                Shortcut::parse(&hotkey)
                    .ok()
                    .map(|shortcut| (shortcut, a.clone()))
            })
            .collect();

        global_handler(sender, a).unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct GapsConfig {
    top: f32,
    right: f32,
    left: f32,
    bottom: f32,
}

impl Default for GapsConfig {
    fn default() -> Self {
        GapsConfig {
            top: 0.,
            right: 0.,
            left: 0.,
            bottom: 0.,
        }
    }
}
